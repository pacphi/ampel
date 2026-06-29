//! Remediation notification seam (Phase 3).
//!
//! The executor emits two events on the happy path — `RemediationRunMerged`
//! (after the consolidated PR merges) and `SourcePrsClosed` (after finalize
//! closes the superseded source PRs). Delivery is abstracted behind
//! [`RemediationNotifier`] so the executor stays testable (a fake records the
//! calls) and CI-safe (the default notifier never touches the network).
//!
//! Production delivery is wired through `ampel-core`'s existing
//! [`ampel_core::services::NotificationService`] via [`SlackNotifier`], which is
//! only constructed when a webhook URL is configured. Payloads carry run/PR
//! identifiers only — never tokens, credentials, or other secrets.

use async_trait::async_trait;
use uuid::Uuid;

use ampel_core::services::{MergeNotificationPayload, MergeResultItem, NotificationService};

/// Emitted after the consolidated PR is merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMergedNotification {
    pub run_id: Uuid,
    pub consolidated_pr_number: i64,
    /// Provider kind (e.g. `github`). Bounded, non-secret.
    pub provider: String,
}

/// Emitted after finalize closes the superseded source PRs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrsClosedNotification {
    pub run_id: Uuid,
    pub consolidated_pr_number: i64,
    pub closed_pr_numbers: Vec<i64>,
}

/// Delivery seam for remediation notifications.
#[async_trait]
pub trait RemediationNotifier: Send + Sync {
    /// A consolidated PR was merged.
    async fn run_merged(&self, event: RunMergedNotification);
    /// The superseded source PRs were closed.
    async fn sources_closed(&self, event: SourcePrsClosedNotification);
}

/// Default notifier: drops every event. Used when no delivery channel is
/// configured (and as the executor's default), so nothing reaches the network.
pub struct NoopNotifier;

#[async_trait]
impl RemediationNotifier for NoopNotifier {
    async fn run_merged(&self, _event: RunMergedNotification) {}
    async fn sources_closed(&self, _event: SourcePrsClosedNotification) {}
}

/// Logs each event via `tracing` (no network). A sensible default for
/// environments without Slack configured.
pub struct LoggingNotifier;

#[async_trait]
impl RemediationNotifier for LoggingNotifier {
    async fn run_merged(&self, event: RunMergedNotification) {
        tracing::info!(
            run_id = %event.run_id,
            consolidated_pr = event.consolidated_pr_number,
            provider = %event.provider,
            "remediation: consolidated PR merged"
        );
    }

    async fn sources_closed(&self, event: SourcePrsClosedNotification) {
        tracing::info!(
            run_id = %event.run_id,
            consolidated_pr = event.consolidated_pr_number,
            closed = ?event.closed_pr_numbers,
            "remediation: source PRs closed"
        );
    }
}

/// Delivers events to Slack via the shared [`NotificationService`].
///
/// Only constructed when a webhook URL is present. Delivery failures are logged,
/// never propagated — a notification problem must not fail a successful run.
pub struct SlackNotifier {
    webhook_url: String,
    channel: Option<String>,
}

impl SlackNotifier {
    pub fn new(webhook_url: String, channel: Option<String>) -> Self {
        Self {
            webhook_url,
            channel,
        }
    }

    async fn send(&self, payload: MergeNotificationPayload) {
        if let Err(e) = NotificationService::send_slack_notification(
            &self.webhook_url,
            self.channel.as_deref(),
            &payload,
        )
        .await
        {
            tracing::warn!(error = %e, "remediation notification delivery failed");
        }
    }
}

#[async_trait]
impl RemediationNotifier for SlackNotifier {
    async fn run_merged(&self, event: RunMergedNotification) {
        let payload = MergeNotificationPayload {
            total: 1,
            success: 1,
            failed: 0,
            skipped: 0,
            results: vec![MergeResultItem {
                repository: event.provider,
                pr_number: event.consolidated_pr_number as i32,
                pr_title: format!("Remediation run {}", event.run_id),
                status: "success".to_string(),
                error: None,
            }],
        };
        self.send(payload).await;
    }

    async fn sources_closed(&self, event: SourcePrsClosedNotification) {
        let results = event
            .closed_pr_numbers
            .iter()
            .map(|n| MergeResultItem {
                repository: format!("run {}", event.run_id),
                pr_number: *n as i32,
                pr_title: format!("Superseded by #{}", event.consolidated_pr_number),
                status: "success".to_string(),
                error: None,
            })
            .collect::<Vec<_>>();
        let count = results.len() as i32;
        let payload = MergeNotificationPayload {
            total: count,
            success: count,
            failed: 0,
            skipped: 0,
            results,
        };
        self.send(payload).await;
    }
}
