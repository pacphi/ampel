//! Playbook resolution + STRICT template rendering (ADR-006).
//!
//! ## 3-tier resolution
//! [`resolve`] selects the effective playbook YAML by precedence
//! **repo-local > DB override > embedded default** (rust-embed), parses it, and
//! then applies the tools-policy CEILING via [`clamp_tools`]: an override may
//! only REMOVE tools relative to the embedded default, never add new ones. The
//! clamp runs after EVERY resolve.
//!
//! ## Instruction rendering (prompt-injection split)
//! [`render_instructions`] renders a task's minijinja template with
//! `UndefinedBehavior::Strict` against a [`PlaybookContext`] that carries ONLY
//! trusted metadata (repo name, branch, failure class). The untrusted values
//! (CI logs, diffs, file contents) are deliberately absent from this context —
//! they are delivered to the model as separate untrusted `context_blocks`, never
//! interpolated into the rendered instruction string. This split is the core
//! prompt-injection defense at the playbook layer.
#![allow(dead_code)] // wired into the worker binary in slice 3

use ampel_core::errors::{AmpelError, AmpelResult};
use minijinja::{context, Environment, UndefinedBehavior};
use rust_embed::RustEmbed;

use super::playbook::{clamp_tools, Playbook, PlaybookTask};

/// Embedded playbook assets (the default ships in the binary).
#[derive(RustEmbed)]
#[folder = "playbooks/"]
struct PlaybookAssets;

/// Resolution scope (informational; the caller selects which DB row maps to a
/// scope before calling [`resolve`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybookScope {
    Repo,
    Org,
    Global,
}

/// The embedded default playbook YAML.
pub fn embedded_default_yaml() -> AmpelResult<String> {
    let asset = PlaybookAssets::get("default.yaml")
        .ok_or_else(|| AmpelError::ConfigError("playbook: embedded default.yaml missing".into()))?;
    String::from_utf8(asset.data.into_owned())
        .map_err(|e| AmpelError::ConfigError(format!("playbook: default.yaml not utf-8: {e}")))
}

/// Trusted metadata available to task templates. Deliberately excludes any
/// untrusted/external content (CI logs, diffs, file contents).
#[derive(Clone, Debug, Default)]
pub struct PlaybookContext {
    pub repo_full_name: String,
    pub base_branch: String,
    pub failure_class: String,
}

/// Resolve the effective playbook for a scope.
///
/// Precedence: `repo_local_yaml` > `db_override_yaml` > embedded default. The
/// tools-policy ceiling (from the embedded default) is applied after parsing so
/// an override can only narrow the allow-list.
pub fn resolve(
    _scope: PlaybookScope,
    repo_local_yaml: Option<&str>,
    db_override_yaml: Option<&str>,
) -> AmpelResult<Playbook> {
    let default_yaml = embedded_default_yaml()?;
    let chosen = repo_local_yaml
        .or(db_override_yaml)
        .unwrap_or(default_yaml.as_str());

    let mut playbook = Playbook::from_yaml(chosen)?;

    // Apply the ceiling from the embedded default (the org/system ceiling).
    let ceiling = Playbook::from_yaml(&default_yaml)?;
    playbook.tools_policy.allowed = clamp_tools(
        &ceiling.tools_policy.allowed,
        &playbook.tools_policy.allowed,
    );

    Ok(playbook)
}

/// Render a task's instruction template under STRICT undefined semantics.
///
/// Any reference to a variable not provided by [`PlaybookContext`] is an error
/// (no silent empty strings). Untrusted data is intentionally not in scope.
pub fn render_instructions(task: &PlaybookTask, ctx: &PlaybookContext) -> AmpelResult<String> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.add_template("task", &task.instructions)
        .map_err(|e| AmpelError::ConfigError(format!("playbook: bad template: {e}")))?;
    let tmpl = env
        .get_template("task")
        .map_err(|e| AmpelError::ConfigError(format!("playbook: template lookup: {e}")))?;
    tmpl.render(context! {
        repo_full_name => ctx.repo_full_name,
        base_branch => ctx.base_branch,
        failure_class => ctx.failure_class,
    })
    .map_err(|e| AmpelError::ConfigError(format!("playbook: render failed: {e}")))
}

/// Build the full trusted `system` instruction: the playbook role followed by
/// the rendered task instructions. Untrusted data is never included here.
pub fn build_system_instruction(
    playbook: &Playbook,
    task: &PlaybookTask,
    ctx: &PlaybookContext,
) -> AmpelResult<String> {
    let rendered = render_instructions(task, ctx)?;
    Ok(format!("{}\n\n{}", playbook.role.trim(), rendered.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::remediation::FailureClass;

    fn ctx() -> PlaybookContext {
        PlaybookContext {
            repo_full_name: "octo/ampel".into(),
            base_branch: "main".into(),
            failure_class: "build_error".into(),
        }
    }

    #[test]
    fn should_resolve_embedded_default_when_no_overrides() {
        let pb = resolve(PlaybookScope::Global, None, None).unwrap();
        assert_eq!(pb.version, 1);
        assert!(pb.tasks.contains_key("failed_ci"));
        assert!(pb.tools_policy.allowed.contains(&"apply_patch".to_string()));
    }

    #[test]
    fn should_prefer_repo_local_over_db_and_embedded() {
        let repo_local = r#"
version: 2
role: "repo role"
tasks:
  failed_ci:
    instructions: "fix {{ repo_full_name }}"
loop:
  max_iterations: 1
  max_seconds: 10
  max_cost_usd: "0.10"
tools_policy:
  allowed: [read_file]
output_contract: unified_diff
"#;
        let db = r#"
version: 3
role: "db role"
tasks:
  failed_ci:
    instructions: "db"
loop:
  max_iterations: 2
  max_seconds: 20
  max_cost_usd: "0.20"
tools_policy:
  allowed: [read_file, write_file]
output_contract: unified_diff
"#;
        let pb = resolve(PlaybookScope::Repo, Some(repo_local), Some(db)).unwrap();
        assert_eq!(pb.version, 2);
        assert_eq!(pb.role, "repo role");
    }

    #[test]
    fn should_clamp_override_tools_to_embedded_ceiling() {
        // Override tries to grant `git_push` (not in the ceiling) — must be dropped,
        // while `read_file`/`apply_patch` (in the ceiling) survive.
        let override_yaml = r#"
role: "r"
tasks:
  failed_ci:
    instructions: "i"
loop:
  max_iterations: 1
  max_seconds: 1
  max_cost_usd: "0.01"
tools_policy:
  allowed: [read_file, apply_patch, git_push]
output_contract: unified_diff
"#;
        let pb = resolve(PlaybookScope::Repo, Some(override_yaml), None).unwrap();
        assert!(pb.tools_policy.allowed.contains(&"read_file".to_string()));
        assert!(pb.tools_policy.allowed.contains(&"apply_patch".to_string()));
        assert!(!pb.tools_policy.allowed.contains(&"git_push".to_string()));
    }

    #[test]
    fn should_render_instructions_with_trusted_metadata() {
        let pb = resolve(PlaybookScope::Global, None, None).unwrap();
        let task = pb.select_task(FailureClass::BuildError).unwrap();
        let rendered = render_instructions(task, &ctx()).unwrap();
        assert!(rendered.contains("octo/ampel"));
        assert!(rendered.contains("main"));
    }

    #[test]
    fn should_error_on_undefined_template_variable_strict() {
        let task = PlaybookTask {
            instructions: "hello {{ unknown_variable }}".into(),
        };
        assert!(render_instructions(&task, &ctx()).is_err());
    }

    #[test]
    fn should_select_lockfile_task_for_lockfile_conflict() {
        let pb = resolve(PlaybookScope::Global, None, None).unwrap();
        let task = pb.select_task(FailureClass::LockfileConflict).unwrap();
        let rendered = render_instructions(task, &ctx()).unwrap();
        assert!(rendered.to_lowercase().contains("lockfile"));
    }

    #[test]
    fn should_build_system_instruction_with_role_then_task() {
        let pb = resolve(PlaybookScope::Global, None, None).unwrap();
        let task = pb.select_task(FailureClass::BuildError).unwrap();
        let system = build_system_instruction(&pb, task, &ctx()).unwrap();
        assert!(system.starts_with("You are an autonomous CI remediation engineer"));
        assert!(system.contains("octo/ampel"));
    }
}
