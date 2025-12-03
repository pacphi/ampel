use crate::models::{AmpelStatus, CICheck, PullRequest, PullRequestWithDetails, Review};

/// Service for pull request-related business logic
pub struct PrService;

impl PrService {
    pub fn new() -> Self {
        Self
    }

    /// Enrich a pull request with status and related data
    pub fn with_details(
        &self,
        pull_request: PullRequest,
        ci_checks: Vec<CICheck>,
        reviews: Vec<Review>,
        repository_owner: String,
        repository_name: String,
    ) -> PullRequestWithDetails {
        let status = AmpelStatus::for_pull_request(&pull_request, &ci_checks, &reviews);

        PullRequestWithDetails {
            pull_request,
            status,
            ci_checks,
            reviews,
            repository_name,
            repository_owner,
        }
    }

    /// Check if a PR author is a known bot
    pub fn is_bot_author(author: &str) -> bool {
        let bot_patterns = [
            "dependabot",
            "renovate",
            "github-actions",
            "greenkeeper",
            "snyk-bot",
            "imgbot",
            "codecov",
            "mergify",
            "stale",
            "allcontributors",
        ];

        let author_lower = author.to_lowercase();
        bot_patterns
            .iter()
            .any(|pattern| author_lower.contains(pattern))
            || author_lower.ends_with("[bot]")
            || author_lower.ends_with("-bot")
    }

    /// Calculate PR age in days
    pub fn age_in_days(pr: &PullRequest) -> i64 {
        let now = chrono::Utc::now();
        (now - pr.created_at).num_days()
    }

    /// Determine if a PR is stale (older than threshold)
    pub fn is_stale(pr: &PullRequest, stale_days: i64) -> bool {
        Self::age_in_days(pr) > stale_days
    }
}

impl Default for PrService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bot_author() {
        assert!(PrService::is_bot_author("dependabot[bot]"));
        assert!(PrService::is_bot_author("renovate[bot]"));
        assert!(PrService::is_bot_author("github-actions[bot]"));
        assert!(PrService::is_bot_author("my-custom-bot"));
        assert!(!PrService::is_bot_author("regular-user"));
        assert!(!PrService::is_bot_author("bot-lover")); // Contains 'bot' but not as suffix
    }
}
