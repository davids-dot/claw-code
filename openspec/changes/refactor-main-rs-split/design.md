## Context

`rusty-claude-cli/src/main.rs` (13,292 lines) contains the entire CLI application logic. It currently imports three sub-modules (init, input, render), but everything else lives in the single file. The file has grown organically as features were added, with no enforced module boundaries.

**Current module layout:**
```
src/
├── init.rs     (~200 lines) — workspace initialization
├── input.rs    (~200 lines) — rustyline REPL editor
├── render.rs   (~200 lines) — terminal rendering, markdown, spinner
└── main.rs     (~13,292 lines) — everything else
```

**Internal structure of main.rs:**
| Region | Lines | Content |
|--------|-------|---------|
| Imports, consts, type aliases | 1-200 | `#![allow]`, `mod`, `use`, `const`, `type` |
| `ModelSource/Provenance` | 60-150 | Model resolution types |
| `main()`, `run()`, helpers | 205-535 | Entry point + CLI action dispatch |
| `CliAction`, `CliOutputFormat` | 506-645 | Action enum definitions |
| `parse_args()` + 50 helpers | 645-1830 | Argument parsing and validation |
| `Doctor` system | 1831-2529 | Health check infrastructure |
| Status/Session functions | 2835-3700 | State snapshot and git parsing |
| `SessionHandle`, `LiveCli`, `BuiltRuntime` structs | 3678-4180 | Core data structures |
| `LiveCli` impl + `run_repl` | 4181-5550 | REPL loop and command dispatch |
| Formatting helpers | 5550-6900 | Report rendering (config, export, diff, etc.) |
| Runtime construction | 6906-7600 | `build_runtime`, plugin/MCP boot |
| `AnthropicRuntimeClient` | 7497-7910 | API client wrapper |
| Output event formatting | 7910-8730 | tool call/result formatting |
| `CliToolExecutor` | 8731-8923 | Tool execution adapter |
| `print_help_to` | 8923-9122 | Help text |
| Tests | 9123-13291 | `#[cfg(test)] mod tests { ... }` |

## Goals / Non-Goals

**Goals:**
- Split `main.rs` into ≤ 2000 lines by moving code into responsibility-focused modules
- Every new file must document its abstraction boundary (via its name, public API surface, and module-level structure)
- Zero behavior change — this is a pure move-and-rename refactoring
- All existing tests pass without modification (except import path updates)
- Each individual migration step compiles independently (no long-lived broken state)

**Non-Goals:**
- No new features or capabilities
- No architectural pattern changes (trait extraction, Command Handler pattern, etc.) — those belong in a future Phase 2
- No changes to `init.rs`, `input.rs`, `render.rs` — existing modules are already well-factored
- No Cargo dependency changes
- No performance optimization
- No public API changes (this is an internal binary crate, not a library)

## Decisions

### D1: Module granularity — fine-grained (one concern per file)

**Chosen:** ~17 new files, each with a single thematic responsibility.

**Rationale:**
- Each file will be 150–1200 lines, well under the 2000-line limit
- File names directly map to the concern they contain (e.g., `doctor.rs` → diagnostic functions)
- Easier to navigate during development — you know exactly which file to open
- The `claw-cli` sibling crate demonstrates this pattern (app.rs, args.rs, init.rs, input.rs, main.rs, render.rs)

**Alternatives considered:**
- *Fewer, larger files* (e.g., 5-6 files at 2000 lines each) — rejected because it just postpones the problem and doesn't improve navigability
- *Single `lib.rs` with sub-modules* — the crate currently has no `lib.rs` and is a standalone binary; adding one would be a larger structural change

### D2: impl LiveCli — concentrate in `live_cli.rs`, REPL commands as free functions in `repl_commands.rs`

**Chosen:** `LiveCli` struct definition + all core methods live in `live_cli.rs`. Command handler functions (`handle_repl_command`, `handle_session_command`, `set_model`, etc.) become free functions in `repl_commands.rs` that accept `&mut LiveCli`.

**Rationale:**
- Rust allows `impl` blocks in multiple files, but this requires careful coordination and makes the struct's API surface hard to discover
- Free functions with an explicit receiver are simpler to understand and maintain
- No orphan rule violations — `repl_commands.rs` functions simply take `cli: &mut LiveCli` as their first parameter

**Implementation pattern:**
```rust
// repl_commands.rs
pub(crate) fn handle_repl_command(
    cli: &mut LiveCli,
    command: SlashCommand,
) -> Result<bool, Box<dyn std::error::Error>> { ... }

pub(crate) fn set_model(
    cli: &mut LiveCli,
    model: Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> { ... }
```

### D3: Visibility strategy — `pub(crate)` for cross-module access, never `pub`

**Chosen:** All functions that need to be called from outside their module are `pub(crate)`. Nothing is `pub` (this is a binary crate, not a library).

**Rationale:**
- `main.rs` is the crate root that re-exports or `use`s sub-modules
- Test code (in `#[cfg(test)] mod tests`) accesses items via `use super::*` — making items `pub(crate)` is sufficient since tests are inline or in `tests/` directory
- Keeps the crate's internal API surface explicit

### D4: Module dependency order — low-level first, then layered

**Chosen:** Build modules in strict dependency order (constants → model → args → ... → live_cli → repl_commands → main).

**Rationale:**
- Avoids circular dependencies between modules
- Each module only depends on modules that are already compiled
- Follows the principle of "stable dependencies": lower-level modules change less frequently

**Dependency graph:**
```
layer 0: constants
layer 1: model
layer 2: permission, progress, doctor
layer 3: args, session
layer 4: status, export, config, diff
layer 5: formatting
layer 6: api_client, tool_executor
layer 7: runtime_builder
layer 8: live_cli
layer 9: repl_commands
layer 10: main (thin dispatch)
```

### D5: Test strategy — inline tests stay with source

**Chosen:** The `#[cfg(test)] mod tests { ... }` block at the bottom of `main.rs` stays attached to `main.rs`. As functions migrate to new modules, their test cases are NOT migrated in Phase 1 — `main.rs` tests continue to import migrated functions via `use super::*` which can reach them through module paths.

**Rationale:**
- Moving tests adds risk of breaking them and increases the scope of each step
- After Phase 1 is complete and stable, Phase 1.5 can migrate tests to their respective source modules
- Rust's module system allows `use crate::module_name::function_name` from within `main.rs` tests
- This minimizes per-step risk

### D6: `main.rs` re-export strategy

**Chosen:** `main.rs` will declare `pub(crate) use module_name::*;` for key modules so existing code (especially tests) can reference items without full path qualification.

**Rationale:**
- Minimizes churn in test code during the migration
- Provides a single reference point for all public crate items
- Can be cleaned up in a follow-up pass

## Module API Surface Summary

| Module | `pub(crate)` items | Dependencies |
|--------|-------------------|--------------|
| `constants.rs` | all `const`, `type` aliases | none |
| `model.rs` | `ModelSource`, `ModelProvenance`, `resolve_model_alias*`, `resolve_repl_model`, `validate_model_syntax`, `config_model_for_current_dir`, `config_alias_for_current_dir`, `max_tokens_for_model` | `constants`, `api` (external) |
| `permission.rs` | `CliPermissionPrompter`, `permission_policy` | `constants`, `runtime` (external) |
| `progress.rs` | `InternalPromptProgressState/Event/Shared/Reporter/Run`, `CliHookProgressReporter`, `describe_tool_progress`, `format_internal_prompt_progress_line` | `constants` |
| `doctor.rs` | `DiagnosticLevel`, `DiagnosticCheck`, `DoctorReport`, `render_doctor_report`, `run_doctor`, `check_*_health` (6 fns), `render_diagnostic_check` | `constants`, `runtime` (external) |
| `args.rs` | `CliAction`, `CliOutputFormat`, `LocalHelpTopic`, `parse_args`, `parse_export_args`, `parse_resume_args`, `parse_system_prompt_args`, `parse_dump_manifests_args`, `parse_local_help_action`, `parse_direct_slash_cli_action`, `parse_acp_args`, `try_resolve_bare_skill_prompt`, `join_optional_args`, `format_unknown_*`, `render_suggestion_line`, `suggest_*`, `levenshtein_distance`, `common_prefix_len`, `normalize_allowed_tools`, `current_tool_registry`, `parse_permission_mode_arg`, `permission_mode_from_label/resolved/default`, `config_permission_mode_for_current_dir`, `provider_label`, `format_connected_line`, `filter_tool_specs`, `is_help_flag`, `parse_single_word_command_alias`, `bare_slash_command_guidance`, `removed_auth_surface_error`, `looks_like_subcommand_typo`, `ranked_suggestions`, `omc_compatibility_note_for_unknown_slash_command` | `constants`, `model`, `runtime`/`tools`/`api` (external) |
| `session.rs` | `SessionHandle`, `ManagedSessionSummary`, `sessions_dir`, `current_session_store`, `new_cli_session`, `create_managed_session_handle`, `resolve_session_reference`, `resolve_managed_session_path`, `list_managed_sessions`, `latest_managed_session`, `load_session_reference`, `delete_managed_session`, `confirm_session_deletion`, `render_session_list`, `format_session_modified_age`, `write_session_clear_backup`, `session_clear_backup_path` | `constants`, `runtime` (external) |
| `status.rs` | `StatusContext`, `StatusUsage`, `GitWorkspaceSummary`, `print_status_snapshot`, `status_json_value`, `status_context`, `format_status_report`, `format_sandbox_report`, `print_sandbox_status_snapshot`, `sandbox_json_value`, `parse_git_status_metadata`, `parse_git_status_branch`, `parse_git_workspace_summary`, `resolve_git_branch_for`, `run_git_capture_in`, `find_git_root_in`, `parse_git_status_metadata_for` | `constants`, `session` |
| `formatting.rs` | `format_tool_call_start`, `format_tool_result`, `format_bash_call/result`, `format_read/write/edit/glob/grep_result`, `format_search_start`, `format_patch_preview`, `format_structured_patch_preview`, `format_generic_tool_result`, `summarize_tool_payload`, `truncate_for_summary`, `truncate_output_for_display`, `render_thinking_block_summary`, `push_output_block`, `response_to_events`, `push_prompt_cache_record`, `prompt_cache_record_to_runtime_event`, `final_assistant_text`, `collect_tool_uses`, `collect_tool_results`, `collect_prompt_cache_events`, `short_tool_id`, `extract_tool_path`, `first_visible_line`, `print_help_to`, `print_help`, `render_help_topic`, `print_help_topic` | `constants` |
| `config.rs` | `render_config_report`, `render_config_json`, `render_memory_report`, `render_memory_json` | `runtime` (external) |
| `diff.rs` | `render_diff_report`, `render_diff_report_for`, `render_diff_json_for`, `run_git_diff_command_in` | none (std + external) |
| `export.rs` | `run_export`, `render_export_text`, `render_session_markdown`, `default_export_filename`, `resolve_export_path`, `summarize_tool_payload_for_markdown` | `session`, `status` |
| `api_client.rs` | `AnthropicRuntimeClient`, `resolve_cli_auth_source`, `resolve_cli_auth_source_for_cwd`, `request_ends_with_tool_result`, `format_user_visible_api_error`, `format_context_window_blocked_error` | `progress`, `api` (external) |
| `tool_executor.rs` | `CliToolExecutor`, `ToolSearchRequest`, `McpToolRequest`, `ListMcpResourcesRequest`, `ReadMcpResourceRequest`, `permission_policy`, `convert_messages` | `formatting`, `runtime`/`tools` (external) |
| `runtime_builder.rs` | `RuntimePluginState`, `RuntimeMcpState`, `BuiltRuntime`, `HookAbortMonitor`, `build_runtime`, `build_runtime_with_plugin_state`, `build_runtime_plugin_state`, `build_runtime_plugin_state_with_loader`, `build_plugin_manager`, `resolve_plugin_path`, `runtime_hook_config_from_plugin_hooks`, `build_system_prompt`, `build_runtime_mcp_state`, `mcp_runtime_tool_definition`, `mcp_wrapper_tool_definitions`, `permission_mode_for_mcp_tool`, `mcp_annotation_flag` | `api_client`, `tool_executor`, `progress`, `permission`, `plugins` (external) |
| `live_cli.rs` | `LiveCli`, `PromptHistoryEntry`, `run_repl`, `run_resume_command`, `resume_session`, `detect_broad_cwd`, `enforce_broad_cwd_policy`, `run_stale_base_preflight` | `runtime_builder`, `session`, `status`, `repl_commands` |
| `repl_commands.rs` | `handle_repl_command`, `handle_session_command`, `handle_plugins_command`, `set_model`, `set_permissions`, `clear_session`, `compact`, `reload_runtime_features`, `print_status`, `print_cost`, `print_prompt_history`, `resume_session`, `export_session`, `print_*` (config/memory/agents/mcp/skills/plugins/diff/version/sandbox), `run_bughunter/ultraplan/teleport/debug_tool_call/commit/pr/issue`, `format_*` (bughunter/ultraplan/pr/issue/commit/auto_compaction/compact/unknown_slash_command_message/model/model_switch/permissions/permissions_switch/cost/resume/compact/history), `render_repl_help`, `render_resume_usage`, `render_teleport_report`, `render_last_tool_debug_report`, `render_prompt_history_report`, `render_version_report`, `collect_session_prompt_history`, `recent_user_context`, `parse_history_count`, `format_history_timestamp`, `civil_from_days`, `git_output`, `git_status_ok`, `command_exists`, `write_temp_text_file`, `sanitize_generated_message`, `parse_titled_body`, `indent_block`, `validate_no_args`, `init_claude_md`, `normalize_permission_mode`, `run_init`, `init_json_value`, `print_acp_status`, `run_worker_state`, `run_mcp_serve`, `print_bootstrap_plan`, `print_system_prompt`, `print_version`, `version_json_value`, `slash_command_completion_candidates_with_sessions`, `truncate_for_prompt` | `live_cli`, `status`, `config`, `diff`, `export`, `doctor`, `commands` (external) |

### `main.rs` (post-split) — what remains

**Items that stay in main.rs:**
```rust
mod init; mod input; mod render;           // existing
mod constants; mod model; mod permission;  // new
mod progress; mod doctor; mod args;
mod session; mod status; mod formatting;
mod config; mod diff; mod export;
mod api_client; mod tool_executor;
mod runtime_builder; mod live_cli;
mod repl_commands;

// Re-exports for test compatibility
pub(crate) use constants::*;
pub(crate) use model::*;
pub(crate) use args::*;
// ... etc for all modules used by tests

fn main() { ... }          // unchanged
fn run() -> ... { ... }    // only dispatches CliAction variants
fn classify_error_kind()   // stays — used in main()
fn split_error_hint()      // stays — used in main()
fn read_piped_stdin()      // stays — used in run()
fn merge_prompt_with_stdin() // stays — used in run()
fn write_mcp_server_fixture() // stays — test utility

// Test module stays here, unchanged, referencing functions via crate:: paths
```

## Risks / Trade-offs

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Missing a function during split** | Medium | High — broken build | Each step compiles independently; CI catches missing items |
| **Circular dependency between modules** | Low | High — redesign needed | Strict dependency ordering (low-level → high-level) prevents this |
| **Test breakage from visibility changes** | Medium | Medium — tests fail | Keep `pub(crate) use module::*` re-exports in main.rs; tests continue to work |
| **Merge conflicts with parallel work** | High (active project) | Medium | Execute refactoring as a focused session; each step is a small diff |
| **Accidental behavior change during move** | Low | High — user-facing bug | Pure copy-paste moves; no logic modification; review each diff carefully |
| **File count increase reduces discoverability** | Medium | Low — developer adaptation | Consistent naming convention; module names map 1:1 to concerns |

## Open Questions

1. **Should `STUB_COMMANDS` (80+ entries) stay in `constants.rs` or be in its own file?** — Currently planned for `constants.rs` (~80 lines of data), can be extracted later if it grows.
2. **Should `write_mcp_server_fixture` be moved to the test support module?** — It's only used in tests. Keep in `main.rs` for now; it's a short function.
3. **How to handle the `#[allow(...)]` attributes?** — The top-level `#![allow(...)]` applies to the crate root. Some module-level allows may need to be duplicated.
