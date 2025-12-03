use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AmpelStatus, GitProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub provider: GitProvider,
    pub provider_id: String,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub state: PullRequestState,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub is_draft: bool,
    pub is_mergeable: Option<bool>,
    pub has_conflicts: bool,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub commits_count: i32,
    pub comments_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub last_synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

impl std::fmt::Display for PullRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullRequestState::Open => write!(f, "open"),
            PullRequestState::Closed => write!(f, "closed"),
            PullRequestState::Merged => write!(f, "merged"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestWithDetails {
    #[serde(flatten)]
    pub pull_request: PullRequest,
    pub status: AmpelStatus,
    pub ci_checks: Vec<CICheck>,
    pub reviews: Vec<Review>,
    pub repository_name: String,
    pub repository_owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CICheck {
    pub id: Uuid,
    pub pull_request_id: Uuid,
    pub name: String,
    pub status: CICheckStatus,
    pub conclusion: Option<CICheckConclusion>,
    pub url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CICheckStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CICheckConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub pull_request_id: Uuid,
    pub reviewer: String,
    pub reviewer_avatar_url: Option<String>,
    pub state: ReviewState,
    pub body: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Pending,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestFilter {
    pub provider: Option<GitProvider>,
    pub status: Option<AmpelStatus>,
    pub author: Option<String>,
    pub reviewer: Option<String>,
    pub repository_id: Option<Uuid>,
    pub is_draft: Option<bool>,
    pub search: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort_by: Option<PullRequestSortField>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestSortField {
    CreatedAt,
    UpdatedAt,
    Title,
    Author,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i32, per_page: i32) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub strategy: MergeStrategy,
    pub commit_title: Option<String>,
    pub commit_message: Option<String>,
    pub delete_branch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Merge,
    #[default]
    Squash,
    Rebase,
}
