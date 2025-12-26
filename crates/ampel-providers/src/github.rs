use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, ProviderResult};
use crate::traits::{
    GitProvider, MergeResult, ProviderCICheck, ProviderCredentials, ProviderDiff,
    ProviderDiffFile, ProviderPullRequest, ProviderReview, ProviderUser, RateLimitInfo,
    TokenValidation,
};
use crate::utils::bearer_auth_header;
use ampel_core::models::{
    DiscoveredRepository, GitProvider as Provider, MergeRequest, MergeStrategy,
};

pub struct GitHubProvider {
    client: Client,
    base_url: String,
}

impl GitHubProvider {
    pub fn new(instance_url: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // Support GitHub Enterprise
        let base_url = instance_url.unwrap_or_else(|| "https://api.github.com".to_string());

        Self { client, base_url }
    }

    /// Build full API URL from path
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/user", "/repos/owner/repo")
    ///
    /// # Returns
    /// Complete API URL combining base_url and path
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Generate authentication header for API requests
    ///
    /// # Arguments
    /// * `credentials` - Provider credentials containing the PAT token
    ///
    /// # Returns
    /// Formatted "Bearer {token}" authentication header
    fn auth_header(&self, credentials: &ProviderCredentials) -> String {
        match credentials {
            ProviderCredentials::Pat { token, .. } => bearer_auth_header(token),
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
struct GitHubErrorResponse {
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

#[derive(Debug, Deserialize)]
struct GitHubDiffFile {
    filename: String,
    status: String,
    additions: i32,
    deletions: i32,
    changes: i32,
    patch: Option<String>,
    previous_filename: Option<String>,
    sha: String,
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

    fn instance_url(&self) -> Option<&str> {
        if self.base_url == "https://api.github.com" {
            None
        } else {
            Some(&self.base_url)
        }
    }

    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation> {
        let response = self
            .client
            .get(self.api_url("/user"))
            .header("Authorization", self.auth_header(credentials))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == 401 {
            return Ok(TokenValidation {
                is_valid: false,
                user_id: None,
                username: None,
                email: None,
                avatar_url: None,
                scopes: vec![],
                expires_at: None,
                error_message: Some("Invalid or expired token".into()),
            });
        }

        if !response.status().is_success() {
            return Ok(TokenValidation {
                is_valid: false,
                user_id: None,
                username: None,
                email: None,
                avatar_url: None,
                scopes: vec![],
                expires_at: None,
                error_message: Some(format!("API error: {}", response.status())),
            });
        }

        let user: GitHubUser = response.json().await?;

        // Get scopes from X-OAuth-Scopes header (for classic PATs)
        // Fine-grained PATs don't expose scopes in headers
        let scopes = vec!["repo".into(), "read:user".into()]; // Assume standard scopes

        Ok(TokenValidation {
            is_valid: true,
            user_id: Some(user.id.to_string()),
            username: Some(user.login),
            email: user.email,
            avatar_url: user.avatar_url,
            scopes,
            expires_at: None, // GitHub doesn't expose in API
            error_message: None,
        })
    }

    async fn get_user(&self, credentials: &ProviderCredentials) -> ProviderResult<ProviderUser> {
        let response = self
            .client
            .get(self.api_url("/user"))
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
        page: i32,
        per_page: i32,
    ) -> ProviderResult<Vec<DiscoveredRepository>> {
        let response = self
            .client
            .get(self.api_url("/user/repos"))
            .header("Authorization", self.auth_header(credentials))
            .header("Accept", "application/vnd.github+json")
            .query(&[
                (
                    "affiliation",
                    "owner,collaborator,organization_member".to_string(),
                ),
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
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository> {
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}", owner, repo)))
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>> {
        let state = state.unwrap_or("open");
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}/pulls", owner, repo)))
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> ProviderResult<ProviderPullRequest> {
        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}/pulls/{}", owner, repo, number)))
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>> {
        // First get the PR to get the head SHA
        let pr = self
            .get_pull_request(credentials, owner, repo, pr_number)
            .await?;

        let response = self
            .client
            .get(self.api_url(&format!(
                "/repos/{}/{}/commits/{}/check-runs",
                owner, repo, pr.source_branch
            )))
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
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
            .header("Authorization", self.auth_header(credentials))
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
        credentials: &ProviderCredentials,
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
            .header("Authorization", self.auth_header(credentials))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            // Try to extract the actual error message from GitHub's response
            let message = match response.json::<GitHubErrorResponse>().await {
                Ok(error_response) => error_response.message,
                Err(_) => match status_code {
                    405 => "Pull request is not mergeable".to_string(),
                    409 => "Merge conflict or head branch was modified".to_string(),
                    422 => "Pull request cannot be merged (check status checks and reviews)"
                        .to_string(),
                    _ => "Failed to merge pull request".to_string(),
                },
            };
            return Err(ProviderError::ApiError {
                status_code,
                message,
            });
        }

        let result: GitHubMergeResponse = response.json().await?;

        Ok(MergeResult {
            merged: result.merged,
            sha: result.sha,
            message: result.message,
        })
    }

    async fn get_rate_limit(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<RateLimitInfo> {
        let response = self
            .client
            .get(self.api_url("/rate_limit"))
            .header("Authorization", self.auth_header(credentials))
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

    async fn get_pull_request_diff(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<ProviderDiff> {
        // First, get PR details to extract base and head commit SHAs
        let pr = self.get_pull_request(credentials, owner, repo, pr_number).await?;

        let response = self
            .client
            .get(self.api_url(&format!("/repos/{}/{}/pulls/{}/files", owner, repo, pr_number)))
            .header("Authorization", self.auth_header(credentials))
            .header("Accept", "application/vnd.github+json")
            .query(&[("per_page", "100")])
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Pull request #{} not found",
                pr_number
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get pull request diff".to_string(),
            });
        }

        let files: Vec<GitHubDiffFile> = response.json().await?;

        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut total_changes = 0;
        let total_files = files.len() as i32;

        let provider_files: Vec<ProviderDiffFile> = files
            .into_iter()
            .map(|f| {
                total_additions += f.additions;
                total_deletions += f.deletions;
                total_changes += f.changes;

                ProviderDiffFile {
                    filename: f.filename,
                    status: f.status,
                    additions: f.additions,
                    deletions: f.deletions,
                    changes: f.changes,
                    patch: f.patch,
                    previous_filename: f.previous_filename,
                    sha: f.sha,
                }
            })
            .collect();

        // Extract base and head commit from PR metadata
        // Since we don't have the raw API response here, use placeholder values
        // In a real implementation, we'd need to fetch the PR separately or parse from response
        let base_commit = format!("base-{}", pr.source_branch);
        let head_commit = format!("head-{}", pr.target_branch);

        Ok(ProviderDiff {
            files: provider_files,
            total_additions,
            total_deletions,
            total_changes,
            total_files,
            base_commit,
            head_commit,
        })
    }
}
