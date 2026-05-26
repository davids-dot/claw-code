use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{json, Map, Value};
use runtime::{ConfigLoader, PermissionMode, ProjectContext, SandboxConfig, SandboxStatus, TokenUsage};
use runtime::resolve_sandbox_status;
use crate::args::*;
use crate::model::*;
use crate::*;

pub(crate) fn parse_git_status_metadata(status: Option<&str>) -> (Option<PathBuf>, Option<String>) {
    parse_git_status_metadata_for(
        &env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        status,
    )
}

pub(crate) fn parse_git_status_branch(status: Option<&str>) -> Option<String> {
    let status = status?;
    let first_line = status.lines().next()?;
    let line = first_line.strip_prefix("## ")?;
    if line.starts_with("HEAD") {
        return Some("detached HEAD".to_string());
    }
    let branch = line.split(['.', ' ']).next().unwrap_or_default().trim();
    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

pub(crate) fn parse_git_workspace_summary(status: Option<&str>) -> GitWorkspaceSummary {
    let mut summary = GitWorkspaceSummary::default();
    let Some(status) = status else {
        return summary;
    };

    for line in status.lines() {
        if line.starts_with("## ") || line.trim().is_empty() {
            continue;
        }

        summary.changed_files += 1;
        let mut chars = line.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');

        if index_status == '?' && worktree_status == '?' {
            summary.untracked_files += 1;
            continue;
        }

        if index_status != ' ' {
            summary.staged_files += 1;
        }
        if worktree_status != ' ' {
            summary.unstaged_files += 1;
        }
        if (matches!(index_status, 'U' | 'A') && matches!(worktree_status, 'U' | 'A'))
            || index_status == 'U'
            || worktree_status == 'U'
        {
            summary.conflicted_files += 1;
        }
    }

    summary
}

pub(crate) fn resolve_git_branch_for(cwd: &Path) -> Option<String> {
    let branch = run_git_capture_in(cwd, &["branch", "--show-current"])?;
    let branch = branch.trim();
    if !branch.is_empty() {
        return Some(branch.to_string());
    }

    let fallback = run_git_capture_in(cwd, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let fallback = fallback.trim();
    if fallback.is_empty() {
        None
    } else if fallback == "HEAD" {
        Some("detached HEAD".to_string())
    } else {
        Some(fallback.to_string())
    }
}

pub(crate) fn run_git_capture_in(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

pub(crate) fn find_git_root_in(cwd: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()?;
    if !output.status.success() {
        return Err("not a git repository".into());
    }
    let path = String::from_utf8(output.stdout)?.trim().to_string();
    if path.is_empty() {
        return Err("empty git root".into());
    }
    Ok(PathBuf::from(path))
}

pub(crate) fn parse_git_status_metadata_for(
    cwd: &Path,
    status: Option<&str>,
) -> (Option<PathBuf>, Option<String>) {
    let branch = resolve_git_branch_for(cwd).or_else(|| parse_git_status_branch(status));
    let project_root = find_git_root_in(cwd).ok();
    (project_root, branch)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn print_status_snapshot(
    model: &str,
    model_flag_raw: Option<&str>,
    permission_mode: PermissionMode,
    output_format: CliOutputFormat,
    allowed_tools: Option<&AllowedToolSet>,
) -> Result<(), Box<dyn std::error::Error>> {
    let usage = StatusUsage {
        message_count: 0,
        turns: 0,
        latest: TokenUsage::default(),
        cumulative: TokenUsage::default(),
        estimated_tokens: 0,
    };
    let context = status_context(None)?;
    // #148: resolve model provenance. If user passed --model, source is
    // "flag" with the raw input preserved. Otherwise probe env -> config
    // -> default and record the winning source.
    let provenance = match model_flag_raw {
        Some(raw) => ModelProvenance {
            resolved: model.to_string(),
            raw: Some(raw.to_string()),
            source: ModelSource::Flag,
        },
        None => ModelProvenance::from_env_or_config_or_default(model),
    };
    match output_format {
        CliOutputFormat::Text => println!(
            "{}",
            format_status_report(
                &provenance.resolved,
                usage,
                permission_mode.as_str(),
                &context,
                Some(&provenance)
            )
        ),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&status_json_value(
                Some(&provenance.resolved),
                usage,
                permission_mode.as_str(),
                &context,
                Some(&provenance),
                allowed_tools,
            ))?
        ),
    }
    Ok(())
}

pub(crate) fn status_json_value(
    model: Option<&str>,
    usage: StatusUsage,
    permission_mode: &str,
    context: &StatusContext,
    // #148: optional provenance for `model` field. Surfaces `model_source`
    // ("flag" | "env" | "config" | "default") and `model_raw` (user input
    // before alias resolution, or null when source is "default"). Callers
    // that don't have provenance (legacy resume paths) pass None, in which
    // case both new fields are omitted.
    provenance: Option<&ModelProvenance>,
    allowed_tools: Option<&AllowedToolSet>,
) -> serde_json::Value {
    // #143: top-level `status` marker so claws can distinguish
    // a clean run from a degraded run (config parse failed but other fields
    // are still populated). `config_load_error` carries the parse-error string
    // when present; it's a string rather than a typed object in Phase 1 and
    // will join the typed-error taxonomy in Phase 2 (ROADMAP §4.44).
    let degraded = context.config_load_error.is_some();
    let model_source = provenance.map(|p| p.source.as_str());
    let model_raw = provenance.and_then(|p| p.raw.clone());
    let allowed_tool_entries = allowed_tools.map(|tools| tools.iter().cloned().collect::<Vec<_>>());
    json!({
        "kind": "status",
        "status": if degraded { "degraded" } else { "ok" },
        "config_load_error": context.config_load_error,
        "model": model,
        "model_source": model_source,
        "model_raw": model_raw,
        "permission_mode": permission_mode,
        "allowed_tools": {
            "source": if allowed_tools.is_some() { "flag" } else { "default" },
            "restricted": allowed_tools.is_some(),
            "entries": allowed_tool_entries,
        },
        "usage": {
            "messages": usage.message_count,
            "turns": usage.turns,
            "latest_total": usage.latest.total_tokens(),
            "cumulative_input": usage.cumulative.input_tokens,
            "cumulative_output": usage.cumulative.output_tokens,
            "cumulative_total": usage.cumulative.total_tokens(),
            "estimated_tokens": usage.estimated_tokens,
        },
        "workspace": {
            "cwd": context.cwd,
            "project_root": context.project_root,
            "git_branch": context.git_branch,
            "git_state": context.git_summary.headline(),
            "changed_files": context.git_summary.changed_files,
            "staged_files": context.git_summary.staged_files,
            "unstaged_files": context.git_summary.unstaged_files,
            "untracked_files": context.git_summary.untracked_files,
            "session": context.session_path.as_ref().map_or_else(|| "live-repl".to_string(), |path| path.display().to_string()),
            "session_id": context.session_path.as_ref().and_then(|path| {
                // Session files are named <session-id>.jsonl directly under
                // .claw/sessions/. Extract the stem (drop the .jsonl extension).
                path.file_stem().map(|n| n.to_string_lossy().into_owned())
            }),
            "loaded_config_files": context.loaded_config_files,
            "discovered_config_files": context.discovered_config_files,
            "memory_file_count": context.memory_file_count,
        },
        "sandbox": {
            "enabled": context.sandbox_status.enabled,
            "active": context.sandbox_status.active,
            "supported": context.sandbox_status.supported,
            "in_container": context.sandbox_status.in_container,
            "requested_namespace": context.sandbox_status.requested.namespace_restrictions,
            "active_namespace": context.sandbox_status.namespace_active,
            "requested_network": context.sandbox_status.requested.network_isolation,
            "active_network": context.sandbox_status.network_active,
            "filesystem_mode": context.sandbox_status.filesystem_mode.as_str(),
            "filesystem_active": context.sandbox_status.filesystem_active,
            "allowed_mounts": context.sandbox_status.allowed_mounts,
            "markers": context.sandbox_status.container_markers,
            "fallback_reason": context.sandbox_status.fallback_reason,
        }
    })
}

pub(crate) fn status_context(
    session_path: Option<&Path>,
) -> Result<StatusContext, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let discovered_config_files = loader.discover().len();
    // #143: degrade gracefully on config parse failure rather than hard-fail.
    // `claw doctor` already does this; `claw status` now matches that contract
    // so that one malformed `mcpServers.*` entry doesn't take down the whole
    // health surface (workspace, git, model, permission, sandbox can still be
    // reported independently).
    let (loaded_config_files, sandbox_status, config_load_error) = match loader.load() {
        Ok(runtime_config) => (
            runtime_config.loaded_entries().len(),
            resolve_sandbox_status(runtime_config.sandbox(), &cwd),
            None,
        ),
        Err(err) => (
            0,
            // Fall back to defaults for sandbox resolution so claws still see
            // a populated sandbox section instead of a missing field. Defaults
            // produce the same output as a runtime config with no sandbox
            // overrides, which is the right degraded-mode shape: we cannot
            // report what the user *intended*, only what is actually in effect.
            resolve_sandbox_status(&runtime::SandboxConfig::default(), &cwd),
            Some(err.to_string()),
        ),
    };
    let project_context = ProjectContext::discover_with_git(&cwd, DEFAULT_DATE)?;
    let (project_root, git_branch) =
        parse_git_status_metadata(project_context.git_status.as_deref());
    let git_summary = parse_git_workspace_summary(project_context.git_status.as_deref());
    let lifecycle = classify_session_lifecycle_for(&cwd);
    Ok(StatusContext {
        cwd,
        session_path: session_path.map(Path::to_path_buf),
        loaded_config_files,
        discovered_config_files,
        memory_file_count: project_context.instruction_files.len(),
        project_root,
        git_branch,
        git_summary,
        session_lifecycle: lifecycle,
        sandbox_status,
        config_load_error,
    })
}

pub(crate) fn classify_session_lifecycle_for(workspace: &Path) -> SessionLifecycleSummary {
    classify_session_lifecycle_from_panes(workspace, discover_tmux_panes())
}

pub(crate) fn classify_session_lifecycle_from_panes(
    workspace: &Path,
    panes: Vec<TmuxPaneSnapshot>,
) -> SessionLifecycleSummary {
    let workspace_dirty = git_worktree_is_dirty(workspace);
    let mut idle_shell = None;
    for pane in panes {
        if !pane_path_matches_workspace(&pane.current_path, workspace) {
            continue;
        }
        if is_idle_shell_command(&pane.current_command) {
            idle_shell.get_or_insert(pane);
        } else {
            return SessionLifecycleSummary {
                kind: SessionLifecycleKind::RunningProcess,
                pane_id: Some(pane.pane_id),
                pane_command: Some(pane.current_command),
                pane_path: Some(pane.current_path),
                workspace_dirty,
                abandoned: false,
            };
        }
    }

    if let Some(pane) = idle_shell {
        SessionLifecycleSummary {
            kind: SessionLifecycleKind::IdleShell,
            pane_id: Some(pane.pane_id),
            pane_command: Some(pane.current_command),
            pane_path: Some(pane.current_path),
            workspace_dirty,
            abandoned: workspace_dirty,
        }
    } else {
        SessionLifecycleSummary {
            kind: SessionLifecycleKind::SavedOnly,
            pane_id: None,
            pane_command: None,
            pane_path: None,
            workspace_dirty,
            abandoned: workspace_dirty,
        }
    }
}

pub(crate) fn discover_tmux_panes() -> Vec<TmuxPaneSnapshot> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_id}\t#{pane_current_command}\t#{pane_current_path}",
        ])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_tmux_pane_snapshots(&stdout)
}

pub(crate) fn parse_tmux_pane_snapshots(output: &str) -> Vec<TmuxPaneSnapshot> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let pane_id = fields.next()?.trim();
            let current_command = fields.next()?.trim();
            let current_path = fields.next()?.trim();
            if pane_id.is_empty() || current_path.is_empty() {
                return None;
            }
            Some(TmuxPaneSnapshot {
                pane_id: pane_id.to_string(),
                current_command: current_command.to_string(),
                current_path: PathBuf::from(current_path),
            })
        })
        .collect()
}

pub(crate) fn pane_path_matches_workspace(pane_path: &Path, workspace: &Path) -> bool {
    let pane_path = fs::canonicalize(pane_path).unwrap_or_else(|_| pane_path.to_path_buf());
    let workspace = fs::canonicalize(workspace).unwrap_or_else(|_| workspace.to_path_buf());
    pane_path == workspace || pane_path.starts_with(&workspace)
}

pub(crate) fn is_idle_shell_command(command: &str) -> bool {
    let command = command.rsplit('/').next().unwrap_or(command);
    matches!(
        command,
        "bash" | "zsh" | "sh" | "fish" | "nu" | "pwsh" | "powershell" | "cmd"
    )
}

pub(crate) fn git_worktree_is_dirty(workspace: &Path) -> bool {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["status", "--porcelain"])
        .output();
    output
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| !output.stdout.is_empty())
}

pub(crate) fn format_status_report(
    model: &str,
    usage: StatusUsage,
    permission_mode: &str,
    context: &StatusContext,
    // #148: optional model provenance to surface in a `Model source` line.
    // Callers without provenance (legacy resume paths) pass None and the
    // source line is omitted for backward compat.
    provenance: Option<&ModelProvenance>,
) -> String {
    // #143: if config failed to parse, surface a degraded banner at the top
    // of the text report so humans see the parse error before the body, while
    // the body below still reports everything that could be resolved without
    // config (workspace, git, sandbox defaults, etc.).
    let status_line = if context.config_load_error.is_some() {
        "Status (degraded)"
    } else {
        "Status"
    };
    let mut blocks: Vec<String> = Vec::new();
    if let Some(err) = context.config_load_error.as_deref() {
        blocks.push(format!(
            "Config load error\n  Status           fail\n  Summary          runtime config failed to load; reporting partial status\n  Details          {err}\n  Hint             `claw doctor` classifies config parse errors; fix the listed field and rerun"
        ));
    }
    // #148: render Model source line after Model, showing where the string
    // came from (flag / env / config / default) and the raw input if any.
    let model_source_line = provenance
        .map(|p| match &p.raw {
            Some(raw) if raw != model => {
                format!("\n  Model source     {} (raw: {raw})", p.source.as_str())
            }
            _ => format!("\n  Model source     {}", p.source.as_str()),
        })
        .unwrap_or_default();
    blocks.extend([
        format!(
            "{status_line}
  Model            {model}{model_source_line}
  Permission mode  {permission_mode}
  Messages         {}
  Turns            {}
  Estimated tokens {}",
            usage.message_count, usage.turns, usage.estimated_tokens,
        ),
        format!(
            "Usage
  Latest total     {}
  Cumulative input {}
  Cumulative output {}
  Cumulative total {}",
            usage.latest.total_tokens(),
            usage.cumulative.input_tokens,
            usage.cumulative.output_tokens,
            usage.cumulative.total_tokens(),
        ),
        format!(
            "Workspace
  Cwd              {}
  Project root     {}
  Git branch       {}
  Git state        {}
  Changed files    {}
  Staged           {}
  Unstaged         {}
  Untracked        {}
  Session          {}
  Config files     loaded {}/{}
  Memory files     {}
  Suggested flow   /status → /diff → /commit",
            context.cwd.display(),
            context
                .project_root
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |path| path.display().to_string()),
            context.git_branch.as_deref().unwrap_or("unknown"),
            context.git_summary.headline(),
            context.git_summary.changed_files,
            context.git_summary.staged_files,
            context.git_summary.unstaged_files,
            context.git_summary.untracked_files,
            context.session_path.as_ref().map_or_else(
                || "live-repl".to_string(),
                |path| path.display().to_string()
            ),
            context.loaded_config_files,
            context.discovered_config_files,
            context.memory_file_count,
        ),
        format_sandbox_report(&context.sandbox_status),
    ]);
    blocks.join("\n\n")
}

pub(crate) fn format_sandbox_report(status: &runtime::SandboxStatus) -> String {
    format!(
        "Sandbox
  Enabled           {}
  Active            {}
  Supported         {}
  In container      {}
  Requested ns      {}
  Active ns         {}
  Requested net     {}
  Active net        {}
  Filesystem mode   {}
  Filesystem active {}
  Allowed mounts    {}
  Markers           {}
  Fallback reason   {}",
        status.enabled,
        status.active,
        status.supported,
        status.in_container,
        status.requested.namespace_restrictions,
        status.namespace_active,
        status.requested.network_isolation,
        status.network_active,
        status.filesystem_mode.as_str(),
        status.filesystem_active,
        if status.allowed_mounts.is_empty() {
            "<none>".to_string()
        } else {
            status.allowed_mounts.join(", ")
        },
        if status.container_markers.is_empty() {
            "<none>".to_string()
        } else {
            status.container_markers.join(", ")
        },
        status
            .fallback_reason
            .clone()
            .unwrap_or_else(|| "<none>".to_string()),
    )
}

pub(crate) fn format_commit_preflight_report(branch: Option<&str>, summary: GitWorkspaceSummary) -> String {
    format!(
        "Commit
  Result           ready
  Branch           {}
  Workspace        {}
  Changed files    {}
  Action           create a git commit from the current workspace changes",
        branch.unwrap_or("unknown"),
        summary.headline(),
        summary.changed_files,
    )
}

pub(crate) fn format_commit_skipped_report() -> String {
    "Commit
  Result           skipped
  Reason           no workspace changes
  Action           create a git commit from the current workspace changes
  Next             /status to inspect context · /diff to inspect repo changes"
        .to_string()
}

pub(crate) fn print_sandbox_status_snapshot(
    output_format: CliOutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader
        .load()
        .unwrap_or_else(|_| runtime::RuntimeConfig::empty());
    let status = resolve_sandbox_status(runtime_config.sandbox(), &cwd);
    match output_format {
        CliOutputFormat::Text => println!("{}", format_sandbox_report(&status)),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&sandbox_json_value(&status))?
        ),
    }
    Ok(())
}

pub(crate) fn sandbox_json_value(status: &runtime::SandboxStatus) -> serde_json::Value {
    json!({
        "kind": "sandbox",
        "enabled": status.enabled,
        "active": status.active,
        "supported": status.supported,
        "in_container": status.in_container,
        "requested_namespace": status.requested.namespace_restrictions,
        "active_namespace": status.namespace_active,
        "requested_network": status.requested.network_isolation,
        "active_network": status.network_active,
        "filesystem_mode": status.filesystem_mode.as_str(),
        "filesystem_active": status.filesystem_active,
        "allowed_mounts": status.allowed_mounts,
        "markers": status.container_markers,
        "fallback_reason": status.fallback_reason,
    })
}

