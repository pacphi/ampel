//! Shared request-security guards for handlers.
//!
//! ## SSRF guard for user-supplied endpoint URLs
//! [`assert_endpoint_safe`] is applied BEFORE any outbound network call (or
//! before persisting a URL that will later drive one) whenever the URL came from
//! a client. It blocks the classic SSRF target set — cloud metadata
//! (`169.254.169.254`), loopback, private (RFC 1918), CGNAT (RFC 6598),
//! link-local, unique-local and unspecified addresses — for **external-egress**
//! providers (Claude/Gemini, or any `egress_class = external`).
//!
//! Local-only providers (Ollama / ONNX, `egress_class = local_only`) are
//! deliberately exempt: reaching `localhost`/a private LAN host is their entire
//! purpose, so the allowance is gated on the provider's egress class rather than
//! applied blanket.
//!
//! ### Resolution depth
//! - Literal IPs are checked directly (no DNS), so metadata/private IP literals
//!   are rejected without touching the network.
//! - Hostnames are resolved to **every** A/AAAA address via the system resolver
//!   (off the async runtime via `spawn_blocking`); if *any* resolved address is
//!   in a blocked range the URL is rejected. This defends against DNS-rebinding
//!   style hostnames (e.g. a public name that resolves to `127.0.0.1`). The HTTP
//!   request to the provider is never attempted when the guard rejects.
//! - IPv4-mapped IPv6 addresses are unwrapped and checked as IPv4.
//!
//! Redirect following in the provider HTTP client is a separate layer; it is not
//! changed here (the providers build their own `reqwest` clients). A follow-up
//! could pin `redirect(Policy::none())` to close the redirect-to-internal vector.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

use ampel_core::remediation::Egress;

use crate::handlers::ApiError;

/// Assert a user-supplied `url_str` is safe to use as a provider endpoint given
/// its `egress` class. Returns a `400`/`422` `ApiError` on rejection and makes
/// no request to the URL itself.
pub async fn assert_endpoint_safe(url_str: &str, egress: Egress) -> Result<(), ApiError> {
    let url =
        url::Url::parse(url_str).map_err(|_| ApiError::bad_request("invalid endpoint_url"))?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(ApiError::bad_request("endpoint_url must use http or https")),
    }

    // Embedded credentials in a URL are a common SSRF/credential-smuggling vector.
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ApiError::bad_request(
            "endpoint_url must not contain userinfo",
        ));
    }

    let host = url
        .host_str()
        .ok_or_else(|| ApiError::bad_request("endpoint_url must include a host"))?;

    // Local-only providers legitimately target localhost / the private LAN.
    if egress == Egress::LocalOnly {
        return Ok(());
    }

    // External egress: a literal IP is checked directly (no DNS, no network).
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(&ip) {
            return Err(ApiError::unprocessable_entity(
                "endpoint_url resolves to a disallowed internal address",
            ));
        }
        return Ok(());
    }

    // Hostname: resolve A/AAAA off the runtime and reject if ANY address is internal.
    let port = url.port_or_known_default().unwrap_or(443);
    let host_owned = host.to_string();
    let addrs: Vec<IpAddr> = tokio::task::spawn_blocking(move || {
        (host_owned.as_str(), port)
            .to_socket_addrs()
            .map(|iter| iter.map(|s| s.ip()).collect::<Vec<_>>())
    })
    .await
    .map_err(|_| ApiError::internal("endpoint resolution failed"))?
    .map_err(|_| ApiError::unprocessable_entity("endpoint_url host could not be resolved"))?;

    if addrs.is_empty() {
        return Err(ApiError::unprocessable_entity(
            "endpoint_url host did not resolve",
        ));
    }
    for ip in addrs {
        if is_blocked_ip(&ip) {
            return Err(ApiError::unprocessable_entity(
                "endpoint_url resolves to a disallowed internal address",
            ));
        }
    }
    Ok(())
}

/// Whether `ip` falls in any range an external-egress call must never reach.
fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => is_blocked_v6(v6),
    }
}

fn is_blocked_v4(v4: &Ipv4Addr) -> bool {
    v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_broadcast()
        || v4.is_documentation()
        || is_cgnat_v4(v4)
        || v4.octets()[0] == 0 // 0.0.0.0/8 "this network"
}

/// RFC 6598 carrier-grade NAT space: 100.64.0.0/10.
fn is_cgnat_v4(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 100 && (o[1] & 0xc0) == 64
}

fn is_blocked_v6(v6: &Ipv6Addr) -> bool {
    if let Some(mapped) = v6.to_ipv4_mapped() {
        return is_blocked_v4(&mapped);
    }
    v6.is_loopback()
        || v6.is_unspecified()
        || is_unique_local_v6(v6) // fc00::/7
        || is_link_local_v6(v6) // fe80::/10
}

fn is_unique_local_v6(v6: &Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(v6: &Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_reject_cloud_metadata_ip_for_external() {
        let r =
            assert_endpoint_safe("http://169.254.169.254/latest/meta-data", Egress::External).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn should_reject_private_ip_for_external() {
        assert!(assert_endpoint_safe("http://10.0.0.1/", Egress::External)
            .await
            .is_err());
        assert!(
            assert_endpoint_safe("http://192.168.1.1/", Egress::External)
                .await
                .is_err()
        );
        assert!(
            assert_endpoint_safe("http://127.0.0.1:8080/", Egress::External)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn should_reject_localhost_hostname_for_external() {
        assert!(
            assert_endpoint_safe("http://localhost:11434/", Egress::External)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn should_allow_localhost_for_local_only() {
        assert!(
            assert_endpoint_safe("http://localhost:11434/", Egress::LocalOnly)
                .await
                .is_ok()
        );
        assert!(assert_endpoint_safe("http://10.0.0.1/", Egress::LocalOnly)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn should_reject_non_http_scheme() {
        assert!(assert_endpoint_safe("file:///etc/passwd", Egress::External)
            .await
            .is_err());
        assert!(
            assert_endpoint_safe("gopher://10.0.0.1/", Egress::LocalOnly)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn should_reject_userinfo() {
        assert!(
            assert_endpoint_safe("http://user:pass@example.com/", Egress::External)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn should_allow_public_ip_for_external() {
        assert!(assert_endpoint_safe("https://1.1.1.1/", Egress::External)
            .await
            .is_ok());
    }
}
