use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, ProviderResult};
use crate::traits::{
    GitProvider, MergeResult, ProviderCICheck, ProviderCredentials,
    ProviderPullRequest, ProviderReview, ProviderUser, RateLimitInfo, TokenValidation,
};
use ampel_core::models::{
    DiscoveredRepository, GitProvider as Provider, MergeRequest, MergeStrategy,
};

pub struct BitbucketProvider {
    client: Client,
    base_url: String,
}

impl BitbucketProvider {
    /// Create a new Bitbucket provider instance
    ///
    /// # Arguments
    /// * `instance_url` - Optional base URL for Bitbucket Server/Data Center.
    ///   Defaults to "https://api.bitbucket.org/2.0" for Bitbucket Cloud
    pub fn new(instance_url: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        let base_url = instance_url.unwrap_or_else(|| "https://api.bitbucket.org/2.0".to_string());

        Self { client, base_url }
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Generate appropriate authentication header for PAT (App Password)
    fn auth_header(&self, credentials: &ProviderCredentials) -> String {
        match credentials {
            ProviderCredentials::Pat { token, username } => {
                // Bitbucket App Passwords require Basic Auth with username:token
                let username = username.as_deref().unwrap_or("");
                let auth = BASE64.encode(format!("{}:{}", username, token));
                format!("Basic {}", auth)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketUser {
    uuid: String,
    username: String,
    display_name: Option<String>,
    links: BitbucketUserLinks,
}

#[derive(Debug, Deserialize)]
struct BitbucketUserLinks {
    avatar: Option<BitbucketLink>,
}

#[derive(Debug, Deserialize)]
struct BitbucketLink {
    href: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketPaginated<T> {
    values: Vec<T>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketRepo {
    uuid: String,
    name: String,
    full_name: String,
    description: Option<String>,
    links: BitbucketRepoLinks,
    mainbranch: Option<BitbucketBranch>,
    is_private: bool,
    owner: BitbucketOwner,
}

#[derive(Debug, Deserialize)]
struct BitbucketRepoLinks {
    html: BitbucketLink,
}

#[derive(Debug, Deserialize)]
struct BitbucketBranch {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketOwner {
    username: Option<String>,
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketPR {
    id: i32,
    title: String,
    description: Option<String>,
    links: BitbucketPRLinks,
    state: String,
    source: BitbucketPRRef,
    destination: BitbucketPRRef,
    author: BitbucketPRAuthor,
    created_on: String,
    updated_on: String,
    merge_commit: Option<BitbucketMergeCommit>,
    closed_by: Option<BitbucketPRAuthor>,
    comment_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPRLinks {
    html: BitbucketLink,
}

#[derive(Debug, Deserialize)]
struct BitbucketPRRef {
    branch: BitbucketBranch,
}

#[derive(Debug, Deserialize)]
struct BitbucketPRAuthor {
    username: Option<String>,
    display_name: Option<String>,
    links: Option<BitbucketUserLinks>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketMergeCommit {
    hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPipeline {
    uuid: String,
    state: BitbucketPipelineState,
    created_on: Option<String>,
    completed_on: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPipelineState {
    name: String,
    result: Option<BitbucketPipelineResult>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPipelineResult {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketApproval {
    user: BitbucketUser,
    date: String,
}

#[derive(Debug, Serialize)]
struct BitbucketMergeRequest {
    message: Option<String>,
    close_source_branch: bool,
    merge_strategy: String,
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
impl GitProvider for BitbucketProvider {
    fn provider_type(&self) -> Provider {
        Provider::Bitbucket
    }

    fn instance_url(&self) -> Option<&str> {
        if self.base_url == "https://api.bitbucket.org/2.0" {
            None // Cloud version
        } else {
            Some(&self.base_url)
        }
    }

    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation> {
        // For PAT with Bitbucket, username is required
        let ProviderCredentials::Pat { username, .. } = credentials;
        if username.is_none() || username.as_ref().unwrap().is_empty() {
            return Ok(TokenValidation {
                is_valid: false,
                user_id: None,
                username: None,
                email: None,
                avatar_url: None,
                scopes: vec![],
                expires_at: None,
                error_message: Some(
                    "Bitbucket App Passwords require a username for Basic Auth".to_string(),
                ),
            });
        }

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
                error_message: Some("Invalid or expired credentials".to_string()),
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

        let user: BitbucketUser = response.json().await?;

        Ok(TokenValidation {
            is_valid: true,
            user_id: Some(user.uuid.clone()),
            username: Some(user.username.clone()),
            email: None, // Bitbucket requires separate API call for email
            avatar_url: user.links.avatar.as_ref().map(|a| a.href.clone()),
            scopes: vec![], // Bitbucket doesn't expose scopes in API response
            expires_at: None, // App Passwords don't expire by default
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

        let user: BitbucketUser = response.json().await?;

        Ok(ProviderUser {
            id: user.uuid,
            username: user.username,
            email: None, // Bitbucket requires separate API call for email
            avatar_url: user.links.avatar.map(|a| a.href),
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
            .get(self.api_url("/repositories"))
            .header("Authorization", self.auth_header(credentials))
            .query(&[
                ("role", "member"),
                ("page", &page.to_string()),
                ("pagelen", &per_page.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list repositories".to_string(),
            });
        }

        let repos: BitbucketPaginated<BitbucketRepo> = response.json().await?;

        Ok(repos
            .values
            .into_iter()
            .map(|r| {
                let parts: Vec<&str> = r.full_name.split('/').collect();
                let (owner, _name) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (
                        r.owner
                            .username
                            .or(r.owner.display_name)
                            .unwrap_or_default(),
                        r.name.clone(),
                    )
                };

                DiscoveredRepository {
                    provider: Provider::Bitbucket,
                    provider_id: r.uuid,
                    owner,
                    name: r.name,
                    full_name: r.full_name,
                    description: r.description,
                    url: r.links.html.href,
                    default_branch: r
                        .mainbranch
                        .map(|b| b.name)
                        .unwrap_or_else(|| "main".to_string()),
                    is_private: r.is_private,
                    is_archived: false, // Bitbucket doesn't have archived concept
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
        let response = self
            .client
            .get(self.api_url(&format!("/repositories/{}/{}", owner, repo)))
            .header("Authorization", self.auth_header(credentials))
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

        let r: BitbucketRepo = response.json().await?;

        Ok(DiscoveredRepository {
            provider: Provider::Bitbucket,
            provider_id: r.uuid,
            owner: owner.to_string(),
            name: r.name,
            full_name: r.full_name,
            description: r.description,
            url: r.links.html.href,
            default_branch: r
                .mainbranch
                .map(|b| b.name)
                .unwrap_or_else(|| "main".to_string()),
            is_private: r.is_private,
            is_archived: false,
        })
    }

    async fn list_pull_requests(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>> {
        let state = state.unwrap_or("OPEN");
        let bb_state = match state {
            "open" => "OPEN",
            "closed" => "MERGED,DECLINED",
            "merged" => "MERGED",
            _ => state,
        };

        let response = self
            .client
            .get(self.api_url(&format!("/repositories/{}/{}/pullrequests", owner, repo)))
            .header("Authorization", self.auth_header(credentials))
            .query(&[("state", bb_state)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list pull requests".to_string(),
            });
        }

        let prs: BitbucketPaginated<BitbucketPR> = response.json().await?;

        Ok(prs
            .values
            .into_iter()
            .map(|pr| {
                let state = match pr.state.as_str() {
                    "OPEN" => "open",
                    "MERGED" => "merged",
                    "DECLINED" | "SUPERSEDED" => "closed",
                    _ => &pr.state,
                };

                ProviderPullRequest {
                    provider_id: pr.id.to_string(),
                    number: pr.id,
                    title: pr.title,
                    description: pr.description,
                    url: pr.links.html.href,
                    state: state.to_string(),
                    source_branch: pr.source.branch.name,
                    target_branch: pr.destination.branch.name,
                    author: pr
                        .author
                        .username
                        .or(pr.author.display_name)
                        .unwrap_or_default(),
                    author_avatar_url: pr.author.links.and_then(|l| l.avatar.map(|a| a.href)),
                    is_draft: false, // Bitbucket doesn't have draft PRs
                    is_mergeable: None,
                    has_conflicts: false,
                    additions: 0,
                    deletions: 0,
                    changed_files: 0,
                    commits_count: 0,
                    comments_count: pr.comment_count.unwrap_or(0),
                    created_at: parse_datetime(&pr.created_on),
                    updated_at: parse_datetime(&pr.updated_on),
                    merged_at: if pr.state == "MERGED" {
                        Some(parse_datetime(&pr.updated_on))
                    } else {
                        None
                    },
                    closed_at: if pr.state != "OPEN" {
                        Some(parse_datetime(&pr.updated_on))
                    } else {
                        None
                    },
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
        let response = self
            .client
            .get(self.api_url(&format!(
                "/repositories/{}/{}/pullrequests/{}",
                owner, repo, number
            )))
            .header("Authorization", self.auth_header(credentials))
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

        let pr: BitbucketPR = response.json().await?;
        let state = match pr.state.as_str() {
            "OPEN" => "open",
            "MERGED" => "merged",
            "DECLINED" | "SUPERSEDED" => "closed",
            _ => &pr.state,
        };

        Ok(ProviderPullRequest {
            provider_id: pr.id.to_string(),
            number: pr.id,
            title: pr.title,
            description: pr.description,
            url: pr.links.html.href,
            state: state.to_string(),
            source_branch: pr.source.branch.name,
            target_branch: pr.destination.branch.name,
            author: pr
                .author
                .username
                .or(pr.author.display_name)
                .unwrap_or_default(),
            author_avatar_url: pr.author.links.and_then(|l| l.avatar.map(|a| a.href)),
            is_draft: false,
            is_mergeable: None,
            has_conflicts: false,
            additions: 0,
            deletions: 0,
            changed_files: 0,
            commits_count: 0,
            comments_count: pr.comment_count.unwrap_or(0),
            created_at: parse_datetime(&pr.created_on),
            updated_at: parse_datetime(&pr.updated_on),
            merged_at: if pr.state == "MERGED" {
                Some(parse_datetime(&pr.updated_on))
            } else {
                None
            },
            closed_at: if pr.state != "OPEN" {
                Some(parse_datetime(&pr.updated_on))
            } else {
                None
            },
        })
    }

    async fn get_ci_checks(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        _pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>> {
        // Get latest pipelines for the repo
        let response = self
            .client
            .get(self.api_url(&format!("/repositories/{}/{}/pipelines/", owner, repo)))
            .header("Authorization", self.auth_header(credentials))
            .query(&[("sort", "-created_on"), ("pagelen", "5")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let pipelines: BitbucketPaginated<BitbucketPipeline> = response.json().await?;

        Ok(pipelines
            .values
            .into_iter()
            .map(|p| {
                let (status, conclusion) = match p.state.name.as_str() {
                    "PENDING" => ("queued".to_string(), None),
                    "IN_PROGRESS" => ("in_progress".to_string(), None),
                    "COMPLETED" => {
                        let conclusion = p.state.result.map(|r| match r.name.as_str() {
                            "SUCCESSFUL" => "success".to_string(),
                            "FAILED" => "failure".to_string(),
                            "STOPPED" => "cancelled".to_string(),
                            _ => "neutral".to_string(),
                        });
                        ("completed".to_string(), conclusion)
                    }
                    _ => ("queued".to_string(), None),
                };

                ProviderCICheck {
                    name: format!("Pipeline {}", p.uuid),
                    status,
                    conclusion,
                    url: None,
                    started_at: parse_datetime_opt(&p.created_on),
                    completed_at: parse_datetime_opt(&p.completed_on),
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
        // Get approvals for the PR
        let response = self
            .client
            .get(self.api_url(&format!(
                "/repositories/{}/{}/pullrequests/{}/activity",
                owner, repo, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        #[derive(Debug, Deserialize)]
        struct Activity {
            approval: Option<BitbucketApproval>,
        }

        let activities: BitbucketPaginated<Activity> = response.json().await?;

        Ok(activities
            .values
            .into_iter()
            .filter_map(|a| {
                a.approval.map(|approval| ProviderReview {
                    id: approval.user.uuid.clone(),
                    reviewer: approval.user.username,
                    reviewer_avatar_url: approval.user.links.avatar.map(|a| a.href),
                    state: "approved".to_string(),
                    body: None,
                    submitted_at: parse_datetime(&approval.date),
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
        let merge_strategy = match merge_request.strategy {
            MergeStrategy::Merge => "merge_commit",
            MergeStrategy::Squash => "squash",
            MergeStrategy::Rebase => "fast_forward",
        };

        let body = BitbucketMergeRequest {
            message: merge_request.commit_message.clone(),
            close_source_branch: merge_request.delete_branch,
            merge_strategy: merge_strategy.to_string(),
        };

        let response = self
            .client
            .post(self.api_url(&format!(
                "/repositories/{}/{}/pullrequests/{}/merge",
                owner, repo, pr_number
            )))
            .header("Authorization", self.auth_header(credentials))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to merge pull request".to_string(),
            });
        }

        Ok(MergeResult {
            merged: true,
            sha: None,
            message: "Merged successfully".to_string(),
        })
    }

    async fn get_rate_limit(&self, _credentials: &ProviderCredentials) -> ProviderResult<RateLimitInfo> {
        // Bitbucket uses different rate limiting approach
        Ok(RateLimitInfo {
            limit: 1000,
            remaining: 1000,
            reset_at: Utc::now() + chrono::Duration::hours(1),
        })
    }
}
