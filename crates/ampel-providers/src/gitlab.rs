use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, ProviderResult};
use crate::traits::{
    GitProvider, MergeResult, ProviderCICheck, ProviderCredentials, ProviderDiff, ProviderDiffFile,
    ProviderPullRequest, ProviderReview, ProviderUser, RateLimitInfo, TokenValidation,
};
use crate::utils::bearer_auth_header;
use ampel_core::models::{
    DiscoveredRepository, GitProvider as Provider, MergeRequest, MergeStrategy,
};

/// GitLab provider implementation for the GitProvider trait
///
/// Supports both GitLab.com and self-hosted GitLab instances
pub struct GitLabProvider {
    client: Client,
    base_url: String,
}

impl GitLabProvider {
    /// Create a new GitLab provider instance
    ///
    /// # Arguments
    /// * `instance_url` - Optional base URL for self-hosted GitLab.
    ///   Defaults to "https://gitlab.com" for GitLab.com
    ///
    /// # Example
    /// ```
    /// use ampel_providers::GitLabProvider;
    ///
    /// // GitLab.com
    /// let provider = GitLabProvider::new(None);
    ///
    /// // Self-hosted GitLab
    /// let self_hosted = GitLabProvider::new(Some("https://gitlab.company.com".into()));
    /// ```
    pub fn new(instance_url: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: instance_url.unwrap_or_else(|| "https://gitlab.com".to_string()),
        }
    }

    /// Build full API URL from path (includes /api/v4 prefix)
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/user", "/projects")
    ///
    /// # Returns
    /// Complete API URL with GitLab's /api/v4 prefix
    fn api_url(&self, path: &str) -> String {
        format!("{}/api/v4{}", self.base_url, path)
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
struct GitLabUser {
    id: i64,
    username: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    id: i64,
    name: String,
    path_with_namespace: String,
    description: Option<String>,
    web_url: String,
    default_branch: Option<String>,
    visibility: String,
    archived: bool,
    namespace: GitLabNamespace,
}

#[derive(Debug, Deserialize)]
struct GitLabNamespace {
    path: String,
}

#[derive(Debug, Deserialize)]
struct GitLabMR {
    id: i64,
    iid: i32,
    title: String,
    description: Option<String>,
    web_url: String,
    state: String,
    source_branch: String,
    target_branch: String,
    author: GitLabMRAuthor,
    draft: bool,
    merge_status: Option<String>,
    has_conflicts: bool,
    changes_count: Option<String>,
    user_notes_count: Option<i32>,
    created_at: String,
    updated_at: String,
    merged_at: Option<String>,
    closed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabMRAuthor {
    username: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabPipeline {
    id: i64,
    status: String,
    web_url: Option<String>,
    created_at: Option<String>,
    finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabJob {
    name: String,
    status: String,
    web_url: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabApproval {
    approved_by: Vec<GitLabApprover>,
}

#[derive(Debug, Deserialize)]
struct GitLabApprover {
    user: GitLabUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabNote {
    id: i64,
    author: GitLabMRAuthor,
    body: String,
    created_at: String,
    #[serde(rename = "type")]
    note_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct GitLabMergeRequest {
    merge_commit_message: Option<String>,
    squash: bool,
    should_remove_source_branch: bool,
}

#[derive(Debug, Deserialize)]
struct GitLabDiffFile {
    old_path: String,
    new_path: String,
    new_file: bool,
    renamed_file: bool,
    deleted_file: bool,
    diff: Option<String>,
    #[serde(default)]
    _a_mode: String,
    #[serde(default)]
    _b_mode: String,
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
impl GitProvider for GitLabProvider {
    fn provider_type(&self) -> Provider {
        Provider::GitLab
    }

    fn instance_url(&self) -> Option<&str> {
        if self.base_url == "https://gitlab.com" {
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
                error_message: Some("Invalid or expired token".to_string()),
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

        let user: GitLabUser = response.json().await?;

        Ok(TokenValidation {
            is_valid: true,
            user_id: Some(user.id.to_string()),
            username: Some(user.username),
            email: user.email,
            avatar_url: user.avatar_url,
            scopes: vec!["api".to_string(), "read_user".to_string()],
            expires_at: None,
            error_message: None,
        })
    }

    async fn get_user(&self, credentials: &ProviderCredentials) -> ProviderResult<ProviderUser> {
        let response = self
            .client
            .get(self.api_url("/user"))
            .header("Authorization", self.auth_header(credentials))
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

        let user: GitLabUser = response.json().await?;

        Ok(ProviderUser {
            id: user.id.to_string(),
            username: user.username,
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
            .get(self.api_url("/projects"))
            .header("Authorization", self.auth_header(credentials))
            .query(&[
                ("membership", "true"),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
                ("order_by", "updated_at"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list projects".to_string(),
            });
        }

        let projects: Vec<GitLabProject> = response.json().await?;

        Ok(projects
            .into_iter()
            .map(|p| {
                let parts: Vec<&str> = p.path_with_namespace.split('/').collect();
                let (owner, name) = if parts.len() >= 2 {
                    (
                        parts[..parts.len() - 1].join("/"),
                        parts.last().unwrap().to_string(),
                    )
                } else {
                    (p.namespace.path.clone(), p.name.clone())
                };

                DiscoveredRepository {
                    provider: Provider::GitLab,
                    provider_id: p.id.to_string(),
                    owner,
                    name,
                    full_name: p.path_with_namespace,
                    description: p.description,
                    url: p.web_url,
                    default_branch: p.default_branch.unwrap_or_else(|| "main".to_string()),
                    is_private: p.visibility != "public",
                    is_archived: p.archived,
                }
            })
            .collect())
    }

    async fn get_repository(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository> {
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        let response = self
            .client
            .get(self.api_url(&format!("/projects/{}", encoded_path)))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Project {}/{} not found",
                owner, repo
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get project".to_string(),
            });
        }

        let p: GitLabProject = response.json().await?;

        Ok(DiscoveredRepository {
            provider: Provider::GitLab,
            provider_id: p.id.to_string(),
            owner: owner.to_string(),
            name: p.name,
            full_name: p.path_with_namespace,
            description: p.description,
            url: p.web_url,
            default_branch: p.default_branch.unwrap_or_else(|| "main".to_string()),
            is_private: p.visibility != "public",
            is_archived: p.archived,
        })
    }

    async fn list_pull_requests(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>> {
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);
        let state = state.unwrap_or("opened");
        let gitlab_state = match state {
            "open" => "opened",
            "closed" => "closed",
            "merged" => "merged",
            _ => state,
        };

        let response = self
            .client
            .get(self.api_url(&format!("/projects/{}/merge_requests", encoded_path)))
            .header("Authorization", self.auth_header(credentials))
            .query(&[("state", gitlab_state), ("per_page", "100")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list merge requests".to_string(),
            });
        }

        let mrs: Vec<GitLabMR> = response.json().await?;

        Ok(mrs
            .into_iter()
            .map(|mr| {
                let state = match mr.state.as_str() {
                    "opened" => "open",
                    _ => &mr.state,
                };
                let changes: i32 = mr.changes_count.and_then(|c| c.parse().ok()).unwrap_or(0);

                ProviderPullRequest {
                    provider_id: mr.id.to_string(),
                    number: mr.iid,
                    title: mr.title,
                    description: mr.description,
                    url: mr.web_url,
                    state: state.to_string(),
                    source_branch: mr.source_branch,
                    target_branch: mr.target_branch,
                    author: mr.author.username,
                    author_avatar_url: mr.author.avatar_url,
                    is_draft: mr.draft,
                    is_mergeable: Some(mr.merge_status.as_deref() == Some("can_be_merged")),
                    has_conflicts: mr.has_conflicts,
                    additions: 0,
                    deletions: 0,
                    changed_files: changes,
                    commits_count: 0,
                    comments_count: mr.user_notes_count.unwrap_or(0),
                    created_at: parse_datetime(&mr.created_at),
                    updated_at: parse_datetime(&mr.updated_at),
                    merged_at: parse_datetime_opt(&mr.merged_at),
                    closed_at: parse_datetime_opt(&mr.closed_at),
                }
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
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        let response = self
            .client
            .get(self.api_url(&format!(
                "/projects/{}/merge_requests/{}",
                encoded_path, number
            )))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Merge request !{} not found",
                number
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get merge request".to_string(),
            });
        }

        let mr: GitLabMR = response.json().await?;
        let state = match mr.state.as_str() {
            "opened" => "open",
            _ => &mr.state,
        };
        let changes: i32 = mr.changes_count.and_then(|c| c.parse().ok()).unwrap_or(0);

        Ok(ProviderPullRequest {
            provider_id: mr.id.to_string(),
            number: mr.iid,
            title: mr.title,
            description: mr.description,
            url: mr.web_url,
            state: state.to_string(),
            source_branch: mr.source_branch,
            target_branch: mr.target_branch,
            author: mr.author.username,
            author_avatar_url: mr.author.avatar_url,
            is_draft: mr.draft,
            is_mergeable: Some(mr.merge_status.as_deref() == Some("can_be_merged")),
            has_conflicts: mr.has_conflicts,
            additions: 0,
            deletions: 0,
            changed_files: changes,
            commits_count: 0,
            comments_count: mr.user_notes_count.unwrap_or(0),
            created_at: parse_datetime(&mr.created_at),
            updated_at: parse_datetime(&mr.updated_at),
            merged_at: parse_datetime_opt(&mr.merged_at),
            closed_at: parse_datetime_opt(&mr.closed_at),
        })
    }

    async fn get_ci_checks(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>> {
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        // Get pipelines for the MR
        let response = self
            .client
            .get(self.api_url(&format!(
                "/projects/{}/merge_requests/{}/pipelines",
                encoded_path, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let pipelines: Vec<GitLabPipeline> = response.json().await?;

        if pipelines.is_empty() {
            return Ok(vec![]);
        }

        // Get jobs for the latest pipeline
        let pipeline_id = pipelines[0].id;
        let response = self
            .client
            .get(self.api_url(&format!(
                "/projects/{}/pipelines/{}/jobs",
                encoded_path, pipeline_id
            )))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let jobs: Vec<GitLabJob> = response.json().await?;

        Ok(jobs
            .into_iter()
            .map(|j| {
                let (status, conclusion) = match j.status.as_str() {
                    "pending" | "created" => ("queued".to_string(), None),
                    "running" => ("in_progress".to_string(), None),
                    "success" => ("completed".to_string(), Some("success".to_string())),
                    "failed" => ("completed".to_string(), Some("failure".to_string())),
                    "canceled" => ("completed".to_string(), Some("cancelled".to_string())),
                    "skipped" => ("completed".to_string(), Some("skipped".to_string())),
                    _ => ("queued".to_string(), None),
                };

                ProviderCICheck {
                    name: j.name,
                    status,
                    conclusion,
                    url: j.web_url,
                    started_at: parse_datetime_opt(&j.started_at),
                    completed_at: parse_datetime_opt(&j.finished_at),
                }
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
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        // Get approvals
        let response = self
            .client
            .get(self.api_url(&format!(
                "/projects/{}/merge_requests/{}/approvals",
                encoded_path, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let approvals: GitLabApproval = response.json().await?;

        Ok(approvals
            .approved_by
            .into_iter()
            .map(|a| ProviderReview {
                id: a.user.id.to_string(),
                reviewer: a.user.username,
                reviewer_avatar_url: a.user.avatar_url,
                state: "approved".to_string(),
                body: None,
                submitted_at: Utc::now(), // GitLab doesn't provide approval time in this endpoint
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
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        let squash = matches!(merge_request.strategy, MergeStrategy::Squash);
        let body = GitLabMergeRequest {
            merge_commit_message: merge_request.commit_message.clone(),
            squash,
            should_remove_source_branch: merge_request.delete_branch,
        };

        let response = self
            .client
            .put(self.api_url(&format!(
                "/projects/{}/merge_requests/{}/merge",
                encoded_path, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .json(&body)
            .send()
            .await?;

        if response.status() == 405 {
            return Err(ProviderError::ApiError {
                status_code: 405,
                message: "Merge request is not mergeable".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to merge request".to_string(),
            });
        }

        Ok(MergeResult {
            merged: true,
            sha: None,
            message: "Merged successfully".to_string(),
        })
    }

    async fn get_rate_limit(
        &self,
        _credentials: &ProviderCredentials,
    ) -> ProviderResult<RateLimitInfo> {
        // GitLab uses different rate limiting approach
        Ok(RateLimitInfo {
            limit: 2000,
            remaining: 2000,
            reset_at: Utc::now() + chrono::Duration::hours(1),
        })
    }

    async fn get_pull_request_diff(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<ProviderDiff> {
        // Get PR details to extract base and head commits
        let pr = self
            .get_pull_request(credentials, owner, repo, pr_number)
            .await?;

        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = urlencoding::encode(&project_path);

        let response = self
            .client
            .get(self.api_url(&format!(
                "/projects/{}/merge_requests/{}/diffs",
                encoded_path, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .query(&[("per_page", "100")])
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Merge request !{} not found",
                pr_number
            )));
        }

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to get merge request diff".to_string(),
            });
        }

        let files: Vec<GitLabDiffFile> = response.json().await?;

        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut total_changes = 0;
        let total_files = files.len() as i32;

        let provider_files: Vec<ProviderDiffFile> = files
            .into_iter()
            .map(|f| {
                let status = if f.new_file {
                    "added"
                } else if f.deleted_file {
                    "removed"
                } else if f.renamed_file {
                    "renamed"
                } else {
                    "modified"
                };

                // Parse diff to count additions/deletions
                let (additions, deletions) = if let Some(ref diff) = f.diff {
                    let mut adds = 0;
                    let mut dels = 0;
                    for line in diff.lines() {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            adds += 1;
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            dels += 1;
                        }
                    }
                    (adds, dels)
                } else {
                    (0, 0)
                };

                total_additions += additions;
                total_deletions += deletions;
                total_changes += additions + deletions;

                // Use b_mode as sha, or generate a simple hash from filename
                let sha = if !f._b_mode.is_empty() {
                    f._b_mode.clone()
                } else {
                    // Generate a simple deterministic hash from the filename
                    let hash: u64 = f
                        .new_path
                        .bytes()
                        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                    format!("{:016x}", hash)
                };

                ProviderDiffFile {
                    filename: f.new_path.clone(),
                    status: status.to_string(),
                    additions,
                    deletions,
                    changes: additions + deletions,
                    patch: f.diff,
                    previous_filename: if f.renamed_file && f.old_path != f.new_path {
                        Some(f.old_path)
                    } else {
                        None
                    },
                    sha,
                }
            })
            .collect();

        // Use branch names as base/head commits (simplified approach)
        let base_commit = pr.target_branch.clone();
        let head_commit = pr.source_branch.clone();

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
