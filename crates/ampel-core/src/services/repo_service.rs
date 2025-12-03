use crate::models::{AmpelStatus, Repository, RepositoryWithStatus};

/// Service for repository-related business logic
pub struct RepoService;

impl RepoService {
    pub fn new() -> Self {
        Self
    }

    /// Enrich a repository with its aggregate status
    pub fn with_status(
        &self,
        repository: Repository,
        pr_statuses: &[AmpelStatus],
    ) -> RepositoryWithStatus {
        let status = AmpelStatus::for_repository(pr_statuses);
        let open_pr_count = pr_statuses.len() as i32;

        RepositoryWithStatus {
            repository,
            status,
            open_pr_count,
        }
    }

    /// Parse a wildcard pattern for repository matching
    /// Supports patterns like "org/*" or "org/repo-*"
    pub fn parse_wildcard_pattern(pattern: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = pattern.split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        let owner = parts[0].to_string();
        let repo_pattern = parts[1].to_string();

        Some((owner, repo_pattern))
    }

    /// Check if a repository name matches a wildcard pattern
    pub fn matches_pattern(repo_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return repo_name.starts_with(prefix);
        }

        if let Some(suffix) = pattern.strip_prefix('*') {
            return repo_name.ends_with(suffix);
        }

        repo_name == pattern
    }
}

impl Default for RepoService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wildcard_pattern() {
        assert_eq!(
            RepoService::parse_wildcard_pattern("myorg/*"),
            Some(("myorg".to_string(), "*".to_string()))
        );

        assert_eq!(
            RepoService::parse_wildcard_pattern("myorg/api-*"),
            Some(("myorg".to_string(), "api-*".to_string()))
        );

        assert_eq!(RepoService::parse_wildcard_pattern("invalid"), None);
    }

    #[test]
    fn test_matches_pattern() {
        assert!(RepoService::matches_pattern("anything", "*"));
        assert!(RepoService::matches_pattern("api-gateway", "api-*"));
        assert!(!RepoService::matches_pattern("web-app", "api-*"));
        assert!(RepoService::matches_pattern("-service", "*-service"));
        assert!(RepoService::matches_pattern("exact-match", "exact-match"));
    }
}
