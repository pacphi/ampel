use ampel_core::models::{
    DiscoveredRepository, GitProvider as Provider, MergeRequest, MergeStrategy,
};
use ampel_providers::error::ProviderError;
use ampel_providers::mock::MockProvider;
use ampel_providers::traits::{
    GitProvider, MergeResult, ProviderCICheck, ProviderCredentials, ProviderPullRequest,
    ProviderReview, ProviderUser, RateLimitInfo, TokenValidation,
};
use chrono::Utc;

fn test_credentials() -> ProviderCredentials {
    ProviderCredentials::Pat {
        token: "test_token_12345".to_string(),
        username: None,
    }
}

#[tokio::test]
async fn test_mock_validate_credentials_success() {
    let validation = TokenValidation {
        is_valid: true,
        user_id: Some("user123".to_string()),
        username: Some("testuser".to_string()),
        email: Some("test@example.com".to_string()),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
        scopes: vec!["repo".to_string(), "read:user".to_string()],
        expires_at: None,
        error_message: None,
    };

    let mock = MockProvider::new().with_validation_result(validation.clone());

    let result = mock
        .validate_credentials(&test_credentials())
        .await
        .unwrap();

    assert!(result.is_valid);
    assert_eq!(result.user_id, Some("user123".to_string()));
    assert_eq!(result.username, Some("testuser".to_string()));
    assert_eq!(result.email, Some("test@example.com".to_string()));
    assert_eq!(result.scopes.len(), 2);
    assert!(result.error_message.is_none());
}

#[tokio::test]
async fn test_mock_validate_credentials_invalid() {
    let validation = TokenValidation {
        is_valid: false,
        user_id: None,
        username: None,
        email: None,
        avatar_url: None,
        scopes: vec![],
        expires_at: None,
        error_message: Some("Invalid token".to_string()),
    };

    let mock = MockProvider::new().with_validation_result(validation);

    let result = mock
        .validate_credentials(&test_credentials())
        .await
        .unwrap();

    assert!(!result.is_valid);
    assert!(result.user_id.is_none());
    assert!(result.username.is_none());
    assert_eq!(result.error_message, Some("Invalid token".to_string()));
}

#[tokio::test]
async fn test_mock_validate_credentials_failure() {
    let mock = MockProvider::new().with_validation_failure();

    let result = mock.validate_credentials(&test_credentials()).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::AuthenticationFailed(msg) => {
            assert_eq!(msg, "Mock validation failure");
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[tokio::test]
async fn test_mock_get_user_success() {
    let user = ProviderUser {
        id: "user123".to_string(),
        username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
    };

    let mock = MockProvider::new().with_user(user.clone());

    let result = mock.get_user(&test_credentials()).await.unwrap();

    assert_eq!(result.id, "user123");
    assert_eq!(result.username, "testuser");
    assert_eq!(result.email, Some("test@example.com".to_string()));
}

#[tokio::test]
async fn test_mock_get_user_failure() {
    let mock = MockProvider::new().with_user_failure();

    let result = mock.get_user(&test_credentials()).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::AuthenticationFailed(msg) => {
            assert_eq!(msg, "Mock user failure");
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[tokio::test]
async fn test_mock_list_repositories() {
    let repos = vec![
        DiscoveredRepository {
            provider: Provider::GitHub,
            provider_id: "12345".to_string(),
            owner: "testorg".to_string(),
            name: "repo1".to_string(),
            full_name: "testorg/repo1".to_string(),
            description: Some("Test repository".to_string()),
            url: "https://github.com/testorg/repo1".to_string(),
            default_branch: "main".to_string(),
            is_private: false,
            is_archived: false,
        },
        DiscoveredRepository {
            provider: Provider::GitHub,
            provider_id: "67890".to_string(),
            owner: "testorg".to_string(),
            name: "repo2".to_string(),
            full_name: "testorg/repo2".to_string(),
            description: None,
            url: "https://github.com/testorg/repo2".to_string(),
            default_branch: "main".to_string(),
            is_private: true,
            is_archived: false,
        },
    ];

    let mock = MockProvider::new().with_repositories(repos.clone());

    let result = mock
        .list_repositories(&test_credentials(), 1, 10)
        .await
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "repo1");
    assert_eq!(result[1].name, "repo2");
    assert!(!result[0].is_private);
    assert!(result[1].is_private);
}

#[tokio::test]
async fn test_mock_list_repositories_empty() {
    let mock = MockProvider::new();

    let result = mock
        .list_repositories(&test_credentials(), 1, 10)
        .await
        .unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_mock_list_repositories_failure() {
    let mock = MockProvider::new().with_repositories_failure();

    let result = mock.list_repositories(&test_credentials(), 1, 10).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::ApiError {
            status_code,
            message,
        } => {
            assert_eq!(status_code, 500);
            assert_eq!(message, "Mock repository failure");
        }
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_mock_get_repository() {
    let repos = vec![DiscoveredRepository {
        provider: Provider::GitHub,
        provider_id: "12345".to_string(),
        owner: "testorg".to_string(),
        name: "repo1".to_string(),
        full_name: "testorg/repo1".to_string(),
        description: Some("Test repository".to_string()),
        url: "https://github.com/testorg/repo1".to_string(),
        default_branch: "main".to_string(),
        is_private: false,
        is_archived: false,
    }];

    let mock = MockProvider::new().with_repositories(repos);

    let result = mock
        .get_repository(&test_credentials(), "testorg", "repo1")
        .await
        .unwrap();

    assert_eq!(result.name, "repo1");
    assert_eq!(result.owner, "testorg");
}

#[tokio::test]
async fn test_mock_get_repository_not_found() {
    let mock = MockProvider::new();

    let result = mock
        .get_repository(&test_credentials(), "testorg", "repo1")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::NotFound(msg) => {
            assert!(msg.contains("testorg/repo1"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_mock_list_pull_requests() {
    let now = Utc::now();

    let prs = vec![
        ProviderPullRequest {
            provider_id: "pr123".to_string(),
            number: 1,
            title: "Add feature X".to_string(),
            description: Some("This adds feature X".to_string()),
            url: "https://github.com/testorg/repo1/pull/1".to_string(),
            state: "open".to_string(),
            source_branch: "feature-x".to_string(),
            target_branch: "main".to_string(),
            author: "developer1".to_string(),
            author_avatar_url: Some("https://example.com/avatar1.png".to_string()),
            is_draft: false,
            is_mergeable: Some(true),
            has_conflicts: false,
            additions: 100,
            deletions: 20,
            changed_files: 5,
            commits_count: 3,
            comments_count: 2,
            created_at: now,
            updated_at: now,
            merged_at: None,
            closed_at: None,
        },
        ProviderPullRequest {
            provider_id: "pr456".to_string(),
            number: 2,
            title: "Fix bug Y".to_string(),
            description: None,
            url: "https://github.com/testorg/repo1/pull/2".to_string(),
            state: "open".to_string(),
            source_branch: "fix-bug-y".to_string(),
            target_branch: "main".to_string(),
            author: "developer2".to_string(),
            author_avatar_url: None,
            is_draft: true,
            is_mergeable: Some(false),
            has_conflicts: true,
            additions: 50,
            deletions: 30,
            changed_files: 2,
            commits_count: 1,
            comments_count: 0,
            created_at: now,
            updated_at: now,
            merged_at: None,
            closed_at: None,
        },
    ];

    let mock = MockProvider::new().with_pull_requests("testorg", "repo1", prs.clone());

    let result = mock
        .list_pull_requests(&test_credentials(), "testorg", "repo1", Some("open"))
        .await
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].number, 1);
    assert_eq!(result[0].title, "Add feature X");
    assert!(!result[0].is_draft);
    assert!(result[1].is_draft);
    assert!(result[1].has_conflicts);
}

#[tokio::test]
async fn test_mock_list_pull_requests_empty() {
    let mock = MockProvider::new();

    let result = mock
        .list_pull_requests(&test_credentials(), "testorg", "repo1", None)
        .await
        .unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_mock_list_pull_requests_failure() {
    let mock = MockProvider::new().with_pull_requests_failure();

    let result = mock
        .list_pull_requests(&test_credentials(), "testorg", "repo1", None)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::ApiError {
            status_code,
            message,
        } => {
            assert_eq!(status_code, 500);
            assert_eq!(message, "Mock pull request failure");
        }
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_mock_get_pull_request() {
    let now = Utc::now();

    let prs = vec![ProviderPullRequest {
        provider_id: "pr123".to_string(),
        number: 42,
        title: "Test PR".to_string(),
        description: Some("Description".to_string()),
        url: "https://github.com/testorg/repo1/pull/42".to_string(),
        state: "open".to_string(),
        source_branch: "feature".to_string(),
        target_branch: "main".to_string(),
        author: "developer".to_string(),
        author_avatar_url: None,
        is_draft: false,
        is_mergeable: Some(true),
        has_conflicts: false,
        additions: 10,
        deletions: 5,
        changed_files: 2,
        commits_count: 1,
        comments_count: 0,
        created_at: now,
        updated_at: now,
        merged_at: None,
        closed_at: None,
    }];

    let mock = MockProvider::new().with_pull_requests("testorg", "repo1", prs);

    let result = mock
        .get_pull_request(&test_credentials(), "testorg", "repo1", 42)
        .await
        .unwrap();

    assert_eq!(result.number, 42);
    assert_eq!(result.title, "Test PR");
}

#[tokio::test]
async fn test_mock_get_pull_request_not_found() {
    let mock = MockProvider::new();

    let result = mock
        .get_pull_request(&test_credentials(), "testorg", "repo1", 999)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::NotFound(msg) => {
            assert!(msg.contains("#999"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_mock_get_ci_checks() {
    let now = Utc::now();

    let checks = vec![
        ProviderCICheck {
            name: "CI Build".to_string(),
            status: "completed".to_string(),
            conclusion: Some("success".to_string()),
            url: Some("https://ci.example.com/build/1".to_string()),
            started_at: Some(now),
            completed_at: Some(now),
        },
        ProviderCICheck {
            name: "Tests".to_string(),
            status: "in_progress".to_string(),
            conclusion: None,
            url: Some("https://ci.example.com/test/1".to_string()),
            started_at: Some(now),
            completed_at: None,
        },
        ProviderCICheck {
            name: "Linting".to_string(),
            status: "completed".to_string(),
            conclusion: Some("failure".to_string()),
            url: None,
            started_at: Some(now),
            completed_at: Some(now),
        },
    ];

    let mock = MockProvider::new().with_ci_checks("testorg", "repo1", 1, checks.clone());

    let result = mock
        .get_ci_checks(&test_credentials(), "testorg", "repo1", 1)
        .await
        .unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "CI Build");
    assert_eq!(result[0].status, "completed");
    assert_eq!(result[0].conclusion, Some("success".to_string()));
    assert_eq!(result[1].status, "in_progress");
    assert!(result[1].conclusion.is_none());
    assert_eq!(result[2].conclusion, Some("failure".to_string()));
}

#[tokio::test]
async fn test_mock_get_ci_checks_empty() {
    let mock = MockProvider::new();

    let result = mock
        .get_ci_checks(&test_credentials(), "testorg", "repo1", 1)
        .await
        .unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_mock_get_reviews() {
    let now = Utc::now();

    let reviews = vec![
        ProviderReview {
            id: "review1".to_string(),
            reviewer: "reviewer1".to_string(),
            reviewer_avatar_url: Some("https://example.com/avatar1.png".to_string()),
            state: "approved".to_string(),
            body: Some("LGTM!".to_string()),
            submitted_at: now,
        },
        ProviderReview {
            id: "review2".to_string(),
            reviewer: "reviewer2".to_string(),
            reviewer_avatar_url: None,
            state: "changes_requested".to_string(),
            body: Some("Please fix the tests".to_string()),
            submitted_at: now,
        },
    ];

    let mock = MockProvider::new().with_reviews("testorg", "repo1", 1, reviews.clone());

    let result = mock
        .get_reviews(&test_credentials(), "testorg", "repo1", 1)
        .await
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].reviewer, "reviewer1");
    assert_eq!(result[0].state, "approved");
    assert_eq!(result[1].state, "changes_requested");
}

#[tokio::test]
async fn test_mock_get_reviews_empty() {
    let mock = MockProvider::new();

    let result = mock
        .get_reviews(&test_credentials(), "testorg", "repo1", 1)
        .await
        .unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_mock_merge_pull_request_success() {
    let merge_result = MergeResult {
        merged: true,
        sha: Some("abc123def456".to_string()),
        message: "Successfully merged".to_string(),
    };

    let mock = MockProvider::new().with_merge_result("testorg", "repo1", 1, merge_result.clone());

    let merge_request = MergeRequest {
        strategy: MergeStrategy::Squash,
        commit_title: Some("Merge PR #1".to_string()),
        commit_message: Some("Merging feature X".to_string()),
        delete_branch: true,
    };

    let result = mock
        .merge_pull_request(&test_credentials(), "testorg", "repo1", 1, &merge_request)
        .await
        .unwrap();

    assert!(result.merged);
    assert_eq!(result.sha, Some("abc123def456".to_string()));
    assert_eq!(result.message, "Successfully merged");
}

#[tokio::test]
async fn test_mock_merge_pull_request_conflict() {
    let error = ProviderError::ApiError {
        status_code: 409,
        message: "Merge conflict detected".to_string(),
    };

    let mock = MockProvider::new().with_merge_failure("testorg", "repo1", 1, error);

    let merge_request = MergeRequest {
        strategy: MergeStrategy::Merge,
        commit_title: None,
        commit_message: None,
        delete_branch: false,
    };

    let result = mock
        .merge_pull_request(&test_credentials(), "testorg", "repo1", 1, &merge_request)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::ApiError {
            status_code,
            message,
        } => {
            assert_eq!(status_code, 409);
            assert!(message.contains("Merge conflict"));
        }
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_mock_merge_pull_request_default() {
    let mock = MockProvider::new();

    let merge_request = MergeRequest {
        strategy: MergeStrategy::Rebase,
        commit_title: None,
        commit_message: None,
        delete_branch: false,
    };

    let result = mock
        .merge_pull_request(&test_credentials(), "testorg", "repo1", 999, &merge_request)
        .await
        .unwrap();

    assert!(result.merged);
    assert!(result.sha.is_some());
    assert!(result.message.contains("successfully merged"));
}

#[tokio::test]
async fn test_mock_get_rate_limit() {
    let now = Utc::now();
    let reset_at = now + chrono::Duration::hours(1);

    let rate_limit = RateLimitInfo {
        limit: 5000,
        remaining: 4500,
        reset_at,
    };

    let mock = MockProvider::new().with_rate_limit(rate_limit.clone());

    let result = mock.get_rate_limit(&test_credentials()).await.unwrap();

    assert_eq!(result.limit, 5000);
    assert_eq!(result.remaining, 4500);
    assert_eq!(result.reset_at, reset_at);
}

#[tokio::test]
async fn test_mock_get_rate_limit_default() {
    let mock = MockProvider::new();

    let result = mock.get_rate_limit(&test_credentials()).await.unwrap();

    assert_eq!(result.limit, 5000);
    assert_eq!(result.remaining, 4999);
    assert!(result.reset_at > Utc::now());
}

#[tokio::test]
async fn test_mock_provider_type_variants() {
    let github = MockProvider::new();
    assert_eq!(github.provider_type(), Provider::GitHub);

    let gitlab = MockProvider::new_with_provider(Provider::GitLab);
    assert_eq!(gitlab.provider_type(), Provider::GitLab);

    let bitbucket = MockProvider::new_with_provider(Provider::Bitbucket);
    assert_eq!(bitbucket.provider_type(), Provider::Bitbucket);
}

#[tokio::test]
async fn test_mock_instance_url() {
    let cloud = MockProvider::new();
    assert_eq!(cloud.instance_url(), None);

    let enterprise = MockProvider::new().with_instance_url("https://git.company.com".to_string());
    assert_eq!(enterprise.instance_url(), Some("https://git.company.com"));
}

#[tokio::test]
async fn test_mock_multiple_configurations() {
    let now = Utc::now();

    let mock = MockProvider::new_with_provider(Provider::GitLab)
        .with_instance_url("https://gitlab.company.com".to_string())
        .with_validation_result(TokenValidation {
            is_valid: true,
            username: Some("testuser".to_string()),
            ..Default::default()
        })
        .with_repositories(vec![DiscoveredRepository {
            provider: Provider::GitLab,
            provider_id: "1".to_string(),
            owner: "org".to_string(),
            name: "repo".to_string(),
            full_name: "org/repo".to_string(),
            description: None,
            url: "https://gitlab.company.com/org/repo".to_string(),
            default_branch: "main".to_string(),
            is_private: false,
            is_archived: false,
        }])
        .with_pull_requests(
            "org",
            "repo",
            vec![ProviderPullRequest {
                provider_id: "mr1".to_string(),
                number: 1,
                title: "Test MR".to_string(),
                description: None,
                url: "https://gitlab.company.com/org/repo/-/merge_requests/1".to_string(),
                state: "opened".to_string(),
                source_branch: "feature".to_string(),
                target_branch: "main".to_string(),
                author: "dev".to_string(),
                author_avatar_url: None,
                is_draft: false,
                is_mergeable: Some(true),
                has_conflicts: false,
                additions: 10,
                deletions: 5,
                changed_files: 1,
                commits_count: 1,
                comments_count: 0,
                created_at: now,
                updated_at: now,
                merged_at: None,
                closed_at: None,
            }],
        );

    assert_eq!(mock.provider_type(), Provider::GitLab);
    assert_eq!(mock.instance_url(), Some("https://gitlab.company.com"));

    let validation = mock
        .validate_credentials(&test_credentials())
        .await
        .unwrap();
    assert!(validation.is_valid);

    let repos = mock
        .list_repositories(&test_credentials(), 1, 10)
        .await
        .unwrap();
    assert_eq!(repos.len(), 1);

    let prs = mock
        .list_pull_requests(&test_credentials(), "org", "repo", None)
        .await
        .unwrap();
    assert_eq!(prs.len(), 1);
}

#[tokio::test]
async fn test_mock_cloneable() {
    let mock1 = MockProvider::new().with_validation_result(TokenValidation {
        is_valid: true,
        username: Some("user1".to_string()),
        ..Default::default()
    });

    let mock2 = mock1.clone();

    let result1 = mock1
        .validate_credentials(&test_credentials())
        .await
        .unwrap();
    let result2 = mock2
        .validate_credentials(&test_credentials())
        .await
        .unwrap();

    assert_eq!(result1.username, result2.username);
}
