use serde::{Deserialize, Serialize};

use super::{CICheck, CICheckConclusion, CICheckStatus, PullRequest, Review, ReviewState};

/// Traffic light status for a PR or repository
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AmpelStatus {
    /// All checks pass + approved + no conflicts - ready to merge
    Green,
    /// Checks pending OR awaiting review
    Yellow,
    /// Checks failed OR conflicts OR blocked
    Red,
    /// No open PRs
    None,
}

impl std::fmt::Display for AmpelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmpelStatus::Green => write!(f, "green"),
            AmpelStatus::Yellow => write!(f, "yellow"),
            AmpelStatus::Red => write!(f, "red"),
            AmpelStatus::None => write!(f, "none"),
        }
    }
}

impl AmpelStatus {
    /// Calculate status for a single PR based on CI checks and reviews
    pub fn for_pull_request(pr: &PullRequest, ci_checks: &[CICheck], reviews: &[Review]) -> Self {
        // Draft PRs are always yellow
        if pr.is_draft {
            return AmpelStatus::Yellow;
        }

        // Conflicts or explicitly not mergeable = red
        if pr.has_conflicts {
            return AmpelStatus::Red;
        }

        if let Some(false) = pr.is_mergeable {
            return AmpelStatus::Red;
        }

        // Check CI status
        let ci_status = Self::evaluate_ci_checks(ci_checks);

        // Check review status
        let review_status = Self::evaluate_reviews(reviews);

        // Combine statuses - worst status wins
        match (ci_status, review_status) {
            (AmpelStatus::Red, _) | (_, AmpelStatus::Red) => AmpelStatus::Red,
            (AmpelStatus::Yellow, _) | (_, AmpelStatus::Yellow) => AmpelStatus::Yellow,
            (AmpelStatus::Green, AmpelStatus::Green) => AmpelStatus::Green,
            (AmpelStatus::None, status) | (status, AmpelStatus::None) => status,
        }
    }

    /// Evaluate CI checks to determine status
    fn evaluate_ci_checks(checks: &[CICheck]) -> AmpelStatus {
        if checks.is_empty() {
            return AmpelStatus::None;
        }

        let mut has_pending = false;
        let mut has_failure = false;

        for check in checks {
            match check.status {
                CICheckStatus::Queued | CICheckStatus::InProgress => {
                    has_pending = true;
                }
                CICheckStatus::Completed => {
                    if let Some(conclusion) = &check.conclusion {
                        match conclusion {
                            CICheckConclusion::Failure
                            | CICheckConclusion::TimedOut
                            | CICheckConclusion::ActionRequired => {
                                has_failure = true;
                            }
                            CICheckConclusion::Cancelled => {
                                has_pending = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if has_failure {
            AmpelStatus::Red
        } else if has_pending {
            AmpelStatus::Yellow
        } else {
            AmpelStatus::Green
        }
    }

    /// Evaluate reviews to determine status
    fn evaluate_reviews(reviews: &[Review]) -> AmpelStatus {
        if reviews.is_empty() {
            return AmpelStatus::Yellow; // Awaiting review
        }

        let mut has_approval = false;
        let mut has_changes_requested = false;

        for review in reviews {
            match review.state {
                ReviewState::Approved => has_approval = true,
                ReviewState::ChangesRequested => has_changes_requested = true,
                _ => {}
            }
        }

        if has_changes_requested {
            AmpelStatus::Red
        } else if has_approval {
            AmpelStatus::Green
        } else {
            AmpelStatus::Yellow
        }
    }

    /// Calculate aggregate status for a repository based on its PRs
    pub fn for_repository(pr_statuses: &[AmpelStatus]) -> Self {
        if pr_statuses.is_empty() {
            return AmpelStatus::None;
        }

        // Repository status is the worst status among all PRs
        let mut has_red = false;
        let mut has_yellow = false;

        for status in pr_statuses {
            match status {
                AmpelStatus::Red => has_red = true,
                AmpelStatus::Yellow => has_yellow = true,
                _ => {}
            }
        }

        if has_red {
            AmpelStatus::Red
        } else if has_yellow {
            AmpelStatus::Yellow
        } else {
            AmpelStatus::Green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_pr() -> PullRequest {
        PullRequest {
            id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            provider: crate::models::GitProvider::GitHub,
            provider_id: "1".to_string(),
            number: 1,
            title: "Test PR".to_string(),
            description: None,
            url: "https://github.com/test/repo/pull/1".to_string(),
            state: crate::models::PullRequestState::Open,
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            author: "testuser".to_string(),
            author_avatar_url: None,
            is_draft: false,
            is_mergeable: Some(true),
            has_conflicts: false,
            additions: 10,
            deletions: 5,
            changed_files: 2,
            commits_count: 1,
            comments_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            merged_at: None,
            closed_at: None,
            last_synced_at: Utc::now(),
        }
    }

    #[test]
    fn test_green_status_with_passing_checks_and_approval() {
        let pr = create_test_pr();
        let checks = vec![CICheck {
            id: Uuid::new_v4(),
            pull_request_id: pr.id,
            name: "build".to_string(),
            status: CICheckStatus::Completed,
            conclusion: Some(CICheckConclusion::Success),
            url: None,
            started_at: None,
            completed_at: None,
            duration_seconds: None,
        }];
        let reviews = vec![Review {
            id: Uuid::new_v4(),
            pull_request_id: pr.id,
            reviewer: "reviewer".to_string(),
            reviewer_avatar_url: None,
            state: ReviewState::Approved,
            body: None,
            submitted_at: Utc::now(),
        }];

        assert_eq!(
            AmpelStatus::for_pull_request(&pr, &checks, &reviews),
            AmpelStatus::Green
        );
    }

    #[test]
    fn test_yellow_status_with_pending_checks() {
        let pr = create_test_pr();
        let checks = vec![CICheck {
            id: Uuid::new_v4(),
            pull_request_id: pr.id,
            name: "build".to_string(),
            status: CICheckStatus::InProgress,
            conclusion: None,
            url: None,
            started_at: None,
            completed_at: None,
            duration_seconds: None,
        }];
        let reviews = vec![Review {
            id: Uuid::new_v4(),
            pull_request_id: pr.id,
            reviewer: "reviewer".to_string(),
            reviewer_avatar_url: None,
            state: ReviewState::Approved,
            body: None,
            submitted_at: Utc::now(),
        }];

        assert_eq!(
            AmpelStatus::for_pull_request(&pr, &checks, &reviews),
            AmpelStatus::Yellow
        );
    }

    #[test]
    fn test_red_status_with_failed_checks() {
        let pr = create_test_pr();
        let checks = vec![CICheck {
            id: Uuid::new_v4(),
            pull_request_id: pr.id,
            name: "build".to_string(),
            status: CICheckStatus::Completed,
            conclusion: Some(CICheckConclusion::Failure),
            url: None,
            started_at: None,
            completed_at: None,
            duration_seconds: None,
        }];
        let reviews = vec![];

        assert_eq!(
            AmpelStatus::for_pull_request(&pr, &checks, &reviews),
            AmpelStatus::Red
        );
    }

    #[test]
    fn test_red_status_with_conflicts() {
        let mut pr = create_test_pr();
        pr.has_conflicts = true;

        assert_eq!(
            AmpelStatus::for_pull_request(&pr, &[], &[]),
            AmpelStatus::Red
        );
    }

    #[test]
    fn test_yellow_status_for_draft() {
        let mut pr = create_test_pr();
        pr.is_draft = true;

        assert_eq!(
            AmpelStatus::for_pull_request(&pr, &[], &[]),
            AmpelStatus::Yellow
        );
    }
}
