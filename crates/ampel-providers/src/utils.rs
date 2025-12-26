/// Utility functions for provider implementations
/// Generate Bearer authentication header for token-based authentication
///
/// # Arguments
/// * `token` - The authentication token to use
///
/// # Returns
/// A formatted "Bearer {token}" authentication header string
///
/// # Example
/// ```
/// use ampel_providers::utils::bearer_auth_header;
/// let header = bearer_auth_header("my-secret-token");
/// assert_eq!(header, "Bearer my-secret-token");
/// ```
pub fn bearer_auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_auth_header() {
        let token = "test-token-123";
        let header = bearer_auth_header(token);
        assert_eq!(header, "Bearer test-token-123");
    }

    #[test]
    fn test_bearer_auth_header_empty() {
        let header = bearer_auth_header("");
        assert_eq!(header, "Bearer ");
    }
}
