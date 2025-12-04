use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Slack error: {0}")]
    Slack(String),
    #[error("Email error: {0}")]
    Email(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeNotificationPayload {
    pub total: i32,
    pub success: i32,
    pub failed: i32,
    pub skipped: i32,
    pub results: Vec<MergeResultItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeResultItem {
    pub repository: String,
    pub pr_number: i32,
    pub pr_title: String,
    pub status: String,
    pub error: Option<String>,
}

pub struct NotificationService;

impl NotificationService {
    /// Send a Slack notification about merge results
    pub async fn send_slack_notification(
        webhook_url: &str,
        channel: Option<&str>,
        payload: &MergeNotificationPayload,
    ) -> Result<(), NotificationError> {
        let status_emoji = if payload.failed == 0 { "✅" } else { "⚠️" };

        let mut text = format!(
            "{} *Bulk Merge Completed*\n\n• Total: {}\n• Merged: {}\n• Failed: {}\n• Skipped: {}",
            status_emoji, payload.total, payload.success, payload.failed, payload.skipped
        );

        // Add details for failed PRs
        let failed_prs: Vec<_> = payload
            .results
            .iter()
            .filter(|r| r.status == "failed")
            .collect();

        if !failed_prs.is_empty() {
            text.push_str("\n\n*Failed PRs:*");
            for pr in failed_prs.iter().take(5) {
                text.push_str(&format!(
                    "\n• {} #{}: {}",
                    pr.repository,
                    pr.pr_number,
                    pr.error.as_deref().unwrap_or("Unknown error")
                ));
            }
            if failed_prs.len() > 5 {
                text.push_str(&format!("\n_...and {} more_", failed_prs.len() - 5));
            }
        }

        let mut message = serde_json::json!({ "text": text });
        if let Some(ch) = channel {
            message["channel"] = serde_json::json!(ch);
        }

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&message)
            .send()
            .await
            .map_err(|e| NotificationError::Slack(e.to_string()))?;

        if !response.status().is_success() {
            return Err(NotificationError::Slack(format!(
                "Webhook returned status {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Send an email notification about merge results
    pub async fn send_email_notification(
        config: &SmtpConfig,
        to_emails: &[String],
        payload: &MergeNotificationPayload,
    ) -> Result<(), NotificationError> {
        use lettre::{
            message::header::ContentType, transport::smtp::authentication::Credentials, Message,
            SmtpTransport, Transport,
        };

        if to_emails.is_empty() {
            return Err(NotificationError::Config(
                "No recipient emails configured".into(),
            ));
        }

        let subject = if payload.failed == 0 {
            format!("Ampel: {} PRs merged successfully", payload.success)
        } else {
            format!(
                "Ampel: Merge completed - {} merged, {} failed",
                payload.success, payload.failed
            )
        };

        let mut body = format!(
            "Bulk Merge Results\n\
             ==================\n\n\
             Total PRs: {}\n\
             Merged: {}\n\
             Failed: {}\n\
             Skipped: {}\n",
            payload.total, payload.success, payload.failed, payload.skipped
        );

        // Add successful merges
        let successful: Vec<_> = payload
            .results
            .iter()
            .filter(|r| r.status == "success")
            .collect();
        if !successful.is_empty() {
            body.push_str("\n\nSuccessfully Merged:\n");
            for pr in &successful {
                body.push_str(&format!(
                    "  - {} #{}: {}\n",
                    pr.repository, pr.pr_number, pr.pr_title
                ));
            }
        }

        // Add failed merges
        let failed: Vec<_> = payload
            .results
            .iter()
            .filter(|r| r.status == "failed")
            .collect();
        if !failed.is_empty() {
            body.push_str("\n\nFailed Merges:\n");
            for pr in &failed {
                body.push_str(&format!(
                    "  - {} #{}: {} - {}\n",
                    pr.repository,
                    pr.pr_number,
                    pr.pr_title,
                    pr.error.as_deref().unwrap_or("Unknown error")
                ));
            }
        }

        body.push_str("\n\n--\nSent by Ampel PR Manager");

        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let mailer = if config.use_tls {
            SmtpTransport::starttls_relay(&config.host)
                .map_err(|e| NotificationError::Email(format!("Invalid SMTP host: {}", e)))?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.host)
                .port(config.port)
                .credentials(creds)
                .build()
        };

        // Send to first recipient (others as CC would require more complex setup)
        let email = Message::builder()
            .from(
                config
                    .from_email
                    .parse()
                    .map_err(|_| NotificationError::Email("Invalid from email".into()))?,
            )
            .to(to_emails[0]
                .parse()
                .map_err(|_| NotificationError::Email("Invalid recipient email".into()))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| NotificationError::Email(format!("Failed to build email: {}", e)))?;

        mailer
            .send(&email)
            .map_err(|e| NotificationError::Email(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}
