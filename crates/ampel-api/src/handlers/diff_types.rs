use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// File status in a diff
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    /// New file created
    Added,
    /// File removed
    Deleted,
    /// File content changed
    Modified,
    /// File moved/renamed
    Renamed,
    /// File copied from another location
    Copied,
}

/// A single file's diff information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffFile {
    /// SHA hash of the file
    pub sha: String,
    /// Original file path (for renames/copies)
    pub old_path: Option<String>,
    /// New file path
    pub new_path: String,
    /// File change status
    pub status: FileStatus,
    /// Number of lines added
    pub additions: i32,
    /// Number of lines deleted
    pub deletions: i32,
    /// Total changes (additions + deletions)
    pub changes: i32,
    /// Unified diff patch
    #[schema(
        example = "@@ -1,7 +1,19 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';"
    )]
    pub patch: String,
}

/// Complete diff response for a pull request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffResponse {
    /// List of changed files with diffs
    pub files: Vec<DiffFile>,
    /// Total lines added across all files
    pub total_additions: i32,
    /// Total lines deleted across all files
    pub total_deletions: i32,
    /// Total number of changed files
    pub total_files: i32,
    /// Base commit SHA
    pub base_commit: String,
    /// Head commit SHA
    pub head_commit: String,
}

/// Metadata about the diff response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffMetadata {
    /// Git provider (github, gitlab, bitbucket)
    pub provider: String,
    /// Whether the response was served from cache
    pub cached: bool,
    /// Age of cached data in seconds (0 if not cached)
    pub cache_age_seconds: i64,
    /// Response timestamp
    pub timestamp: String,
}

/// Complete API response with diff data and metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffApiResponse {
    /// Success indicator
    pub success: bool,
    /// Diff data
    pub data: DiffResponse,
    /// Response metadata
    pub metadata: DiffMetadata,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffError {
    /// Error code
    #[schema(example = "PR_NOT_FOUND")]
    pub code: String,
    /// Human-readable error message
    #[schema(example = "Pull request not found or access denied")]
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiffErrorResponse {
    /// Success indicator (always false for errors)
    pub success: bool,
    /// Error information
    pub error: DiffError,
}

/// Query parameters for diff endpoint
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DiffQuery {
    /// Diff format (unified or split)
    #[serde(default = "default_format")]
    #[schema(example = "unified")]
    pub format: String,
    /// Number of context lines around changes
    #[serde(default = "default_context")]
    #[schema(example = 3)]
    pub context: i32,
}

fn default_format() -> String {
    "unified".to_string()
}

fn default_context() -> i32 {
    3
}
