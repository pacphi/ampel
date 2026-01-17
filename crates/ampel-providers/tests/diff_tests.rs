//! Unit tests for git diff provider transformations
//!
//! These tests verify the diff transformation logic for GitHub, GitLab, and Bitbucket
//! without making actual API calls. They test:
//! - Provider-specific diff format transformation
//! - Unified model normalization
//! - Language detection logic
//! - Status value mapping across providers
//!
//! ## Running These Tests
//!
//! ```bash
//! # Run all provider tests
//! cargo test -p ampel-providers
//!
//! # Run only diff tests
//! cargo test -p ampel-providers --test diff_tests
//! ```

/// Mock diff data structure representing provider-specific diff format
#[derive(Debug, Clone)]
pub struct ProviderDiff {
    pub file_path: String,
    pub status: String,
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub patch: Option<String>,
    pub sha: Option<String>,
    pub previous_filename: Option<String>,
}

/// Unified diff model normalized across all providers
#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedDiff {
    pub file_path: String,
    pub status: DiffStatus,
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub patch: Option<String>,
    pub language: Option<String>,
    pub is_binary: bool,
    pub previous_filename: Option<String>,
}

/// Normalized diff status across providers
#[derive(Debug, Clone, PartialEq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Unchanged,
}

/// GitHub-specific status values
mod github {
    pub const STATUS_ADDED: &str = "added";
    pub const STATUS_MODIFIED: &str = "modified";
    pub const STATUS_REMOVED: &str = "removed";
    pub const STATUS_RENAMED: &str = "renamed";
    pub const STATUS_COPIED: &str = "copied";
    pub const STATUS_UNCHANGED: &str = "unchanged";
}

/// GitLab-specific status values
mod gitlab {
    pub const STATUS_NEW: &str = "new";
    pub const STATUS_MODIFIED: &str = "modified";
    pub const STATUS_DELETED: &str = "deleted";
    pub const STATUS_RENAMED: &str = "renamed";
}

/// Bitbucket-specific status values
mod bitbucket {
    pub const STATUS_ADDED: &str = "ADDED";
    pub const STATUS_MODIFIED: &str = "MODIFIED";
    pub const STATUS_REMOVED: &str = "REMOVED";
    pub const STATUS_MOVED: &str = "MOVED";
}

/// Transform GitHub diff to unified model
pub fn transform_github_diff(diff: ProviderDiff) -> UnifiedDiff {
    let status = match diff.status.as_str() {
        github::STATUS_ADDED => DiffStatus::Added,
        github::STATUS_MODIFIED => DiffStatus::Modified,
        github::STATUS_REMOVED => DiffStatus::Deleted,
        github::STATUS_RENAMED => DiffStatus::Renamed,
        github::STATUS_COPIED => DiffStatus::Copied,
        github::STATUS_UNCHANGED => DiffStatus::Unchanged,
        _ => DiffStatus::Modified,
    };

    let language = detect_language(&diff.file_path);
    let is_binary = is_binary_file(&diff.file_path);

    UnifiedDiff {
        file_path: diff.file_path,
        status,
        additions: diff.additions,
        deletions: diff.deletions,
        changes: diff.changes,
        patch: diff.patch,
        language,
        is_binary,
        previous_filename: diff.previous_filename,
    }
}

/// Transform GitLab diff to unified model
pub fn transform_gitlab_diff(diff: ProviderDiff) -> UnifiedDiff {
    let status = match diff.status.as_str() {
        gitlab::STATUS_NEW => DiffStatus::Added,
        gitlab::STATUS_MODIFIED => DiffStatus::Modified,
        gitlab::STATUS_DELETED => DiffStatus::Deleted,
        gitlab::STATUS_RENAMED => DiffStatus::Renamed,
        _ => DiffStatus::Modified,
    };

    let language = detect_language(&diff.file_path);
    let is_binary = is_binary_file(&diff.file_path);

    UnifiedDiff {
        file_path: diff.file_path,
        status,
        additions: diff.additions,
        deletions: diff.deletions,
        changes: diff.changes,
        patch: diff.patch,
        language,
        is_binary,
        previous_filename: diff.previous_filename,
    }
}

/// Transform Bitbucket diff to unified model
pub fn transform_bitbucket_diff(diff: ProviderDiff) -> UnifiedDiff {
    let status = match diff.status.as_str() {
        bitbucket::STATUS_ADDED => DiffStatus::Added,
        bitbucket::STATUS_MODIFIED => DiffStatus::Modified,
        bitbucket::STATUS_REMOVED => DiffStatus::Deleted,
        bitbucket::STATUS_MOVED => DiffStatus::Renamed,
        _ => DiffStatus::Modified,
    };

    let language = detect_language(&diff.file_path);
    let is_binary = is_binary_file(&diff.file_path);

    UnifiedDiff {
        file_path: diff.file_path,
        status,
        additions: diff.additions,
        deletions: diff.deletions,
        changes: diff.changes,
        patch: diff.patch,
        language,
        is_binary,
        previous_filename: diff.previous_filename,
    }
}

/// Detect programming language from file extension
pub fn detect_language(file_path: &str) -> Option<String> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())?;

    let language = match extension.to_lowercase().as_str() {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "py" => "Python",
        "go" => "Go",
        "java" => "Java",
        "cpp" | "cc" | "cxx" => "C++",
        "c" | "h" => "C",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "cs" => "C#",
        "scala" => "Scala",
        "sh" | "bash" => "Shell",
        "sql" => "SQL",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "markdown" => "Markdown",
        _ => return None,
    };

    Some(language.to_string())
}

/// Check if file is binary based on extension
pub fn is_binary_file(file_path: &str) -> bool {
    let extension = match std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some(ext) => ext.to_lowercase(),
        None => return false,
    };

    matches!(
        extension.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "bmp"
            | "ico"
            | "pdf"
            | "zip"
            | "tar"
            | "gz"
            | "bin"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "wasm"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // GitHub Diff Transformation Tests
    #[test]
    fn test_github_added_file() {
        let diff = ProviderDiff {
            file_path: "src/main.rs".to_string(),
            status: github::STATUS_ADDED.to_string(),
            additions: 50,
            deletions: 0,
            changes: 50,
            patch: Some("@@ -0,0 +1,50 @@".to_string()),
            sha: Some("abc123".to_string()),
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);

        assert_eq!(unified.status, DiffStatus::Added);
        assert_eq!(unified.file_path, "src/main.rs");
        assert_eq!(unified.additions, 50);
        assert_eq!(unified.deletions, 0);
        assert_eq!(unified.language, Some("Rust".to_string()));
        assert!(!unified.is_binary);
    }

    #[test]
    fn test_github_modified_file() {
        let diff = ProviderDiff {
            file_path: "frontend/App.tsx".to_string(),
            status: github::STATUS_MODIFIED.to_string(),
            additions: 15,
            deletions: 8,
            changes: 23,
            patch: Some("@@ -10,8 +10,15 @@".to_string()),
            sha: Some("def456".to_string()),
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);

        assert_eq!(unified.status, DiffStatus::Modified);
        assert_eq!(unified.language, Some("TypeScript".to_string()));
        assert_eq!(unified.additions, 15);
        assert_eq!(unified.deletions, 8);
    }

    #[test]
    fn test_github_deleted_file() {
        let diff = ProviderDiff {
            file_path: "old/legacy.js".to_string(),
            status: github::STATUS_REMOVED.to_string(),
            additions: 0,
            deletions: 100,
            changes: 100,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);

        assert_eq!(unified.status, DiffStatus::Deleted);
        assert_eq!(unified.deletions, 100);
        assert_eq!(unified.additions, 0);
    }

    #[test]
    fn test_github_renamed_file() {
        let diff = ProviderDiff {
            file_path: "src/new_name.py".to_string(),
            status: github::STATUS_RENAMED.to_string(),
            additions: 5,
            deletions: 3,
            changes: 8,
            patch: Some("@@ -1,3 +1,5 @@".to_string()),
            sha: Some("ghi789".to_string()),
            previous_filename: Some("src/old_name.py".to_string()),
        };

        let unified = transform_github_diff(diff);

        assert_eq!(unified.status, DiffStatus::Renamed);
        assert_eq!(
            unified.previous_filename,
            Some("src/old_name.py".to_string())
        );
        assert_eq!(unified.language, Some("Python".to_string()));
    }

    // GitLab Diff Transformation Tests
    #[test]
    fn test_gitlab_new_file() {
        let diff = ProviderDiff {
            file_path: "api/handler.go".to_string(),
            status: gitlab::STATUS_NEW.to_string(),
            additions: 80,
            deletions: 0,
            changes: 80,
            patch: Some("diff --git a/api/handler.go".to_string()),
            sha: None,
            previous_filename: None,
        };

        let unified = transform_gitlab_diff(diff);

        assert_eq!(unified.status, DiffStatus::Added);
        assert_eq!(unified.language, Some("Go".to_string()));
    }

    #[test]
    fn test_gitlab_modified_file() {
        let diff = ProviderDiff {
            file_path: "config.yaml".to_string(),
            status: gitlab::STATUS_MODIFIED.to_string(),
            additions: 3,
            deletions: 2,
            changes: 5,
            patch: Some("@@ -5,2 +5,3 @@".to_string()),
            sha: None,
            previous_filename: None,
        };

        let unified = transform_gitlab_diff(diff);

        assert_eq!(unified.status, DiffStatus::Modified);
        assert_eq!(unified.language, Some("YAML".to_string()));
    }

    #[test]
    fn test_gitlab_deleted_file() {
        let diff = ProviderDiff {
            file_path: "deprecated/old.rb".to_string(),
            status: gitlab::STATUS_DELETED.to_string(),
            additions: 0,
            deletions: 45,
            changes: 45,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let unified = transform_gitlab_diff(diff);

        assert_eq!(unified.status, DiffStatus::Deleted);
        assert_eq!(unified.language, Some("Ruby".to_string()));
    }

    #[test]
    fn test_gitlab_renamed_file() {
        let diff = ProviderDiff {
            file_path: "models/user_v2.kt".to_string(),
            status: gitlab::STATUS_RENAMED.to_string(),
            additions: 10,
            deletions: 5,
            changes: 15,
            patch: Some("diff --git a/models/user.kt".to_string()),
            sha: None,
            previous_filename: Some("models/user.kt".to_string()),
        };

        let unified = transform_gitlab_diff(diff);

        assert_eq!(unified.status, DiffStatus::Renamed);
        assert_eq!(unified.language, Some("Kotlin".to_string()));
        assert_eq!(
            unified.previous_filename,
            Some("models/user.kt".to_string())
        );
    }

    // Bitbucket Diff Transformation Tests
    #[test]
    fn test_bitbucket_added_file() {
        let diff = ProviderDiff {
            file_path: "service/auth.java".to_string(),
            status: bitbucket::STATUS_ADDED.to_string(),
            additions: 120,
            deletions: 0,
            changes: 120,
            patch: Some("@@ -0,0 +1,120 @@".to_string()),
            sha: Some("jkl012".to_string()),
            previous_filename: None,
        };

        let unified = transform_bitbucket_diff(diff);

        assert_eq!(unified.status, DiffStatus::Added);
        assert_eq!(unified.language, Some("Java".to_string()));
    }

    #[test]
    fn test_bitbucket_modified_file() {
        let diff = ProviderDiff {
            file_path: "styles/main.scss".to_string(),
            status: bitbucket::STATUS_MODIFIED.to_string(),
            additions: 25,
            deletions: 15,
            changes: 40,
            patch: Some("@@ -100,15 +100,25 @@".to_string()),
            sha: Some("mno345".to_string()),
            previous_filename: None,
        };

        let unified = transform_bitbucket_diff(diff);

        assert_eq!(unified.status, DiffStatus::Modified);
        assert_eq!(unified.language, Some("SCSS".to_string()));
    }

    #[test]
    fn test_bitbucket_removed_file() {
        let diff = ProviderDiff {
            file_path: "test/obsolete_test.cpp".to_string(),
            status: bitbucket::STATUS_REMOVED.to_string(),
            additions: 0,
            deletions: 200,
            changes: 200,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let unified = transform_bitbucket_diff(diff);

        assert_eq!(unified.status, DiffStatus::Deleted);
        assert_eq!(unified.language, Some("C++".to_string()));
    }

    #[test]
    fn test_bitbucket_moved_file() {
        let diff = ProviderDiff {
            file_path: "lib/utils/helper.cs".to_string(),
            status: bitbucket::STATUS_MOVED.to_string(),
            additions: 2,
            deletions: 1,
            changes: 3,
            patch: Some("@@ -1,1 +1,2 @@".to_string()),
            sha: Some("pqr678".to_string()),
            previous_filename: Some("utils/helper.cs".to_string()),
        };

        let unified = transform_bitbucket_diff(diff);

        assert_eq!(unified.status, DiffStatus::Renamed);
        assert_eq!(unified.language, Some("C#".to_string()));
        assert_eq!(
            unified.previous_filename,
            Some("utils/helper.cs".to_string())
        );
    }

    // Language Detection Tests
    #[test]
    fn test_detect_rust_language() {
        assert_eq!(detect_language("src/main.rs"), Some("Rust".to_string()));
    }

    #[test]
    fn test_detect_typescript_language() {
        assert_eq!(
            detect_language("components/App.tsx"),
            Some("TypeScript".to_string())
        );
        assert_eq!(
            detect_language("utils/helper.ts"),
            Some("TypeScript".to_string())
        );
    }

    #[test]
    fn test_detect_javascript_language() {
        assert_eq!(detect_language("index.js"), Some("JavaScript".to_string()));
        assert_eq!(
            detect_language("Component.jsx"),
            Some("JavaScript".to_string())
        );
    }

    #[test]
    fn test_detect_python_language() {
        assert_eq!(detect_language("script.py"), Some("Python".to_string()));
    }

    #[test]
    fn test_detect_config_languages() {
        assert_eq!(detect_language("config.yaml"), Some("YAML".to_string()));
        assert_eq!(detect_language("data.json"), Some("JSON".to_string()));
        assert_eq!(detect_language("Cargo.toml"), Some("TOML".to_string()));
    }

    #[test]
    fn test_detect_unknown_language() {
        assert_eq!(detect_language("random.xyz"), None);
        assert_eq!(detect_language("no_extension"), None);
    }

    #[test]
    fn test_case_insensitive_detection() {
        assert_eq!(detect_language("File.RS"), Some("Rust".to_string()));
        assert_eq!(detect_language("Config.YAML"), Some("YAML".to_string()));
    }

    // Binary File Detection Tests
    #[test]
    fn test_binary_image_files() {
        assert!(is_binary_file("logo.png"));
        assert!(is_binary_file("photo.jpg"));
        assert!(is_binary_file("icon.gif"));
        assert!(is_binary_file("background.jpeg"));
    }

    #[test]
    fn test_binary_archive_files() {
        assert!(is_binary_file("archive.zip"));
        assert!(is_binary_file("backup.tar"));
        assert!(is_binary_file("compressed.gz"));
    }

    #[test]
    fn test_binary_executable_files() {
        assert!(is_binary_file("program.exe"));
        assert!(is_binary_file("library.dll"));
        assert!(is_binary_file("unix.so"));
        assert!(is_binary_file("mac.dylib"));
        assert!(is_binary_file("module.wasm"));
    }

    #[test]
    fn test_text_files_not_binary() {
        assert!(!is_binary_file("source.rs"));
        assert!(!is_binary_file("script.py"));
        assert!(!is_binary_file("config.yaml"));
        assert!(!is_binary_file("README.md"));
        assert!(!is_binary_file("style.css"));
    }

    #[test]
    fn test_case_insensitive_binary_detection() {
        assert!(is_binary_file("Image.PNG"));
        assert!(is_binary_file("Archive.ZIP"));
    }

    // Status Mapping Tests
    #[test]
    fn test_status_normalization_across_providers() {
        // Test that "added" status maps correctly from all providers
        let github_add = ProviderDiff {
            file_path: "test.rs".to_string(),
            status: github::STATUS_ADDED.to_string(),
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let gitlab_add = ProviderDiff {
            file_path: "test.rs".to_string(),
            status: gitlab::STATUS_NEW.to_string(),
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let bitbucket_add = ProviderDiff {
            file_path: "test.rs".to_string(),
            status: bitbucket::STATUS_ADDED.to_string(),
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        assert_eq!(transform_github_diff(github_add).status, DiffStatus::Added);
        assert_eq!(transform_gitlab_diff(gitlab_add).status, DiffStatus::Added);
        assert_eq!(
            transform_bitbucket_diff(bitbucket_add).status,
            DiffStatus::Added
        );
    }

    #[test]
    fn test_unknown_status_defaults_to_modified() {
        let diff = ProviderDiff {
            file_path: "test.rs".to_string(),
            status: "unknown_status".to_string(),
            additions: 5,
            deletions: 3,
            changes: 8,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        assert_eq!(
            transform_github_diff(diff.clone()).status,
            DiffStatus::Modified
        );
        assert_eq!(
            transform_gitlab_diff(diff.clone()).status,
            DiffStatus::Modified
        );
        assert_eq!(transform_bitbucket_diff(diff).status, DiffStatus::Modified);
    }

    // Edge Cases
    #[test]
    fn test_empty_patch() {
        let diff = ProviderDiff {
            file_path: "empty.rs".to_string(),
            status: github::STATUS_MODIFIED.to_string(),
            additions: 0,
            deletions: 0,
            changes: 0,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);
        assert_eq!(unified.patch, None);
    }

    #[test]
    fn test_large_diff_counts() {
        let diff = ProviderDiff {
            file_path: "generated.json".to_string(),
            status: github::STATUS_MODIFIED.to_string(),
            additions: 5000,
            deletions: 3000,
            changes: 8000,
            patch: Some("Large patch...".to_string()),
            sha: Some("large123".to_string()),
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);
        assert_eq!(unified.additions, 5000);
        assert_eq!(unified.deletions, 3000);
        assert_eq!(unified.changes, 8000);
    }

    #[test]
    fn test_special_characters_in_filename() {
        let diff = ProviderDiff {
            file_path: "path/with spaces/file-name_v2.rs".to_string(),
            status: github::STATUS_ADDED.to_string(),
            additions: 10,
            deletions: 0,
            changes: 10,
            patch: None,
            sha: None,
            previous_filename: None,
        };

        let unified = transform_github_diff(diff);
        assert_eq!(unified.file_path, "path/with spaces/file-name_v2.rs");
        assert_eq!(unified.language, Some("Rust".to_string()));
    }
}
