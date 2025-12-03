use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, ProviderResult};
use crate::traits::{
    GitProvider, MergeResult, OAuthToken, ProviderCICheck, ProviderPullRequest, ProviderReview,
    ProviderUser, RateLimitInfo,
};
use ampel_core::models::{
    DiscoveredRepository, GitProvider as Provider, MergeRequest, MergeStrategy,
};

pub struct GitHubProvider {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    base_url: String,
}

impl GitHubProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            client_id,
            client_secret,
            redirect_uri,
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// Create a provider with a custom base URL (for GitHub Enterprise or PAT-only auth)
    pub fn new_with_base_url(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // For GitHub Enterprise, the API URL is typically /api/v3 on the base
        let base_url = base_url.map_or_else(
            || "https://api.github.com".to_string(),
            |url| {
                if url.contains("api.github.com") {
                    url
                } else {
                    // GitHub Enterprise Server API path
                    format!("{}/api/v3", url.trim_end_matches('/'))
                }
            },
        );

        Self {
            client,
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            base_url,
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    id: i64,
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    default_branch: String,
    private: bool,
    archived: bool,
    owner: GitHubRepoOwner,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPR {
    id: i64,
    number: i32,
    title: String,
    body: Option<String>,
    html_url: String,
    state: String,
    draft: Option<bool>,
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
    head: GitHubPRBranch,
    base: GitHubPRBranch,
    user: GitHubPRUser,
    additions: Option<i32>,
    deletions: Option<i32>,
    changed_files: Option<i32>,
    commits: Option<i32>,
    comments: Option<i32>,
    created_at: String,
    updated_at: String,
    merged_at: Option<String>,
    closed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubPRBranch {
    #[serde(rename = "ref")]
    branch_ref: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPRUser {
    login: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubCheckRun {
    name: String,
    status: String,
    conclusion: Option<String>,
    html_url: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubCheckRunsResponse {
    check_runs: Vec<GitHubCheckRun>,
}

#[derive(Debug, Deserialize)]
struct GitHubReview {
    id: i64,
    user: GitHubPRUser,
    state: String,
    body: Option<String>,
    submitted_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct GitHubMergeRequest {
    commit_title: Option<String>,
    commit_message: Option<String>,
    merge_method: String,
}

#[derive(Debug, Deserialize)]
struct GitHubMergeResponse {
    merged: bool,
    sha: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRateLimit {
    resources: GitHubRateLimitResources,
}

#[derive(Debug, Deserialize)]
struct GitHubRateLimitResources {
    core: GitHubRateLimitCore,
}

#[derive(Debug, Deserialize)]
struct GitHubRateLimitCore {
    limit: i32,
    remaining: i32,
    reset: i64,
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_datetime_opt(s: &Option<String>) -> Option<DateTime<Utc>> {
    s.as_ref().map(|s| parse_datetime(s))
}

#[async_trait]
impl GitProvider for GitHubProvider {
    fn provider_type(&self) -> Provider {
        Provider::GitHub
    }

    fn get_oauth_url(&self, state: &str) -> String {
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=repo%20read:user%20user:email&state={}",
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            state
        )
    }

    async fn exchange_code(&self, code: &str) -> ProviderResult<OAuthToken> {
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("code", &code.to_string()),
                ("redirect_uri", &self.redirect_uri),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::AuthenticationFailed(
                "Failed to exchange code".to_string(),
            ));
        }

        let token: GitHubTokenResponse = response.json().await?;
        let expires_at = token
            .expires_in
            .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

        Ok(OAuthToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            token_type: token.token_type,
            expires_at,
            scopes: token
                .scope
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> ProviderResult<OAuthToken> {
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("grant_type", &"refresh_token".to_string()),
                ("refresh_token", &refresh_token.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::AuthenticationFailed(
                "Failed to refresh token".to_string(),
            ));
        }

        let token: GitHubTokenResponse = response.json().await?;
        let expires_at = token
            .expires_in
            .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

        Ok(OAuthToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            token_type: token.token_type,
            expires_at,
            scopes: token
                .scope
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }

    async fn get_user(&self, access_token: &str) -> ProviderResult<ProviderUser> {
        let response = self
            .client
            .get(self.api_url("/user"))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == 401 {
            return Err(ProviderError::AuthenticationFailed(
                "Invalid token".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get user".to_string(),
            });
        }

        let user: GitHubUser = response.json().await?;

        Ok(ProviderUser {
            id: user.id.to_string(),
            username: user.login,
            email: user.email,
            avatar_url: user.avatar_url,
        })
    }

    async fn list_repositories(
        &self,
        access_token: &str,
        page: i32,
        per_page: i32,
    ) -> ProviderResult<Vec<DiscoveredRepository>> {
        let response = self
            .client
            .get(self.api_url("/user/repos"))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .query(&[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
                ("sort", "updated".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list repositories".to_string(),
            });
        }

        let repos: Vec<GitHubRepo> = response.json().await?;

        Ok(repos
            .into_iter()
            .map(|r| DiscoveredRepository {
                provider: Provider::GitHub,
                provider_id: r.id.to_string(),
                owner: r.owner.login,
                name: r.name,
                full_name: r.full_name,
                description: r.description,
                url: r.html_url,
                default_branch: r.default_branch,
                is_private: r.private,
                is_archived: r.archived,
            })
            .collect())
    }

    async fn get_repository(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository> {
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}", owner, repo)))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Repository {}/{} not found",
                owner, repo
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get repository".to_string(),
            });
        }

        let r: GitHubRepo = response.json().await?;

        Ok(DiscoveredRepository {
            provider: Provider::GitHub,
            provider_id: r.id.to_string(),
            owner: r.owner.login,
            name: r.name,
            full_name: r.full_name,
            description: r.description,
            url: r.html_url,
            default_branch: r.default_branch,
            is_private: r.private,
            is_archived: r.archived,
        })
    }

    async fn list_pull_requests(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>> {
        let state = state.unwrap_or("open");
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}/pulls", owner, repo)))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .query(&[("state", state), ("per_page", "100")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list pull requests".to_string(),
            });
        }

        let prs: Vec<GitHubPR> = response.json().await?;

        Ok(prs
            .into_iter()
            .map(|pr| ProviderPullRequest {
                provider_id: pr.id.to_string(),
                number: pr.number,
                title: pr.title,
                description: pr.body,
                url: pr.html_url,
                state: pr.state,
                source_branch: pr.head.branch_ref,
                target_branch: pr.base.branch_ref,
                author: pr.user.login,
                author_avatar_url: pr.user.avatar_url,
                is_draft: pr.draft.unwrap_or(false),
                is_mergeable: pr.mergeable,
                has_conflicts: pr.mergeable_state.as_deref() == Some("dirty"),
                additions: pr.additions.unwrap_or(0),
                deletions: pr.deletions.unwrap_or(0),
                changed_files: pr.changed_files.unwrap_or(0),
                commits_count: pr.commits.unwrap_or(0),
                comments_count: pr.comments.unwrap_or(0),
                created_at: parse_datetime(&pr.created_at),
                updated_at: parse_datetime(&pr.updated_at),
                merged_at: parse_datetime_opt(&pr.merged_at),
                closed_at: parse_datetime_opt(&pr.closed_at),
            })
            .collect())
    }

    async fn get_pull_request(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> ProviderResult<ProviderPullRequest> {
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}/pulls/{}", owner, repo, number)))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Pull request #{} not found",
                number
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get pull request".to_string(),
            });
        }

        let pr: GitHubPR = response.json().await?;

        Ok(ProviderPullRequest {
            provider_id: pr.id.to_string(),
            number: pr.number,
            title: pr.title,
            description: pr.body,
            url: pr.html_url,
            state: pr.state,
            source_branch: pr.head.branch_ref,
            target_branch: pr.base.branch_ref,
            author: pr.user.login,
            author_avatar_url: pr.user.avatar_url,
            is_draft: pr.draft.unwrap_or(false),
            is_mergeable: pr.mergeable,
            has_conflicts: pr.mergeable_state.as_deref() == Some("dirty"),
            additions: pr.additions.unwrap_or(0),
            deletions: pr.deletions.unwrap_or(0),
            changed_files: pr.changed_files.unwrap_or(0),
            commits_count: pr.commits.unwrap_or(0),
            comments_count: pr.comments.unwrap_or(0),
            created_at: parse_datetime(&pr.created_at),
            updated_at: parse_datetime(&pr.updated_at),
            merged_at: parse_datetime_opt(&pr.merged_at),
            closed_at: parse_datetime_opt(&pr.closed_at),
        })
    }

    async fn get_ci_checks(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>> {
        // First get the PR to get the head SHA
        let pr = self
            .get_pull_request(access_token, owner, repo, pr_number)
            .await?;

        let response = self
            .client
            .get(self.api_url(&format!(
                "/repos/{}/{}/commits/{}/check-runs",
                owner, repo, pr.source_branch
            )))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            // If check runs fail, return empty list (might not have any checks)
            return Ok(vec![]);
        }

        let checks: GitHubCheckRunsResponse = response.json().await?;

        Ok(checks
            .check_runs
            .into_iter()
            .map(|c| ProviderCICheck {
                name: c.name,
                status: c.status,
                conclusion: c.conclusion,
                url: c.html_url,
                started_at: parse_datetime_opt(&c.started_at),
                completed_at: parse_datetime_opt(&c.completed_at),
            })
            .collect())
    }

    async fn get_reviews(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderReview>> {
        let response = self
            .client
            .get(self.api_url(&format!(
                "/repos/{}/{}/pulls/{}/reviews",
                owner, repo, pr_number
            )))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get reviews".to_string(),
            });
        }

        let reviews: Vec<GitHubReview> = response.json().await?;

        Ok(reviews
            .into_iter()
            .filter_map(|r| {
                r.submitted_at.map(|submitted_at| ProviderReview {
                    id: r.id.to_string(),
                    reviewer: r.user.login,
                    reviewer_avatar_url: r.user.avatar_url,
                    state: r.state.to_lowercase(),
                    body: r.body,
                    submitted_at: parse_datetime(&submitted_at),
                })
            })
            .collect())
    }

    async fn merge_pull_request(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
        merge_request: &MergeRequest,
    ) -> ProviderResult<MergeResult> {
        let merge_method = match merge_request.strategy {
            MergeStrategy::Merge => "merge",
            MergeStrategy::Squash => "squash",
            MergeStrategy::Rebase => "rebase",
        };

        let body = GitHubMergeRequest {
            commit_title: merge_request.commit_title.clone(),
            commit_message: merge_request.commit_message.clone(),
            merge_method: merge_method.to_string(),
        };

        let response = self
            .client
            .put(self.api_url(&format!(
                "/repos/{}/{}/pulls/{}/merge",
                owner, repo, pr_number
            )))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .await?;

        if response.status() == 405 {
            return Err(ProviderError::ApiError {
                status_code: 405,
                message: "Pull request is not mergeable".to_string(),
            });
        }

        if response.status() == 409 {
            return Err(ProviderError::ApiError {
                status_code: 409,
                message: "Merge conflict or head branch was modified".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to merge pull request".to_string(),
            });
        }

        let result: GitHubMergeResponse = response.json().await?;

        Ok(MergeResult {
            merged: result.merged,
            sha: result.sha,
            message: result.message,
        })
    }

    async fn get_rate_limit(&self, access_token: &str) -> ProviderResult<RateLimitInfo> {
        let response = self
            .client
            .get(self.api_url("/rate_limit"))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get rate limit".to_string(),
            });
        }

        let rate: GitHubRateLimit = response.json().await?;

        Ok(RateLimitInfo {
            limit: rate.resources.core.limit,
            remaining: rate.resources.core.remaining,
            reset_at: Utc.timestamp_opt(rate.resources.core.reset, 0).unwrap(),
        })
    }
}
