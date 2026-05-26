## 1. Create constants.rs — extract all const and type aliases

- [x] 1.1 Create `src/constants.rs` with all `const` definitions (DEFAULT_MODEL, VERSION, STUB_COMMANDS, etc.) and type aliases (AllowedToolSet, RuntimePluginStateBuildOutput) extracted from `main.rs` lines 62-200
- [x] 1.2 Add `mod constants;` to `main.rs`; keep original items as `pub(crate) use constants::*;` re-exports so existing code continues to compile
- [x] 1.3 Run `cargo build` and `cargo test` — must pass with zero errors

## 2. Create model.rs — extract model resolution types and functions

- [x] 2.1 Create `src/model.rs` with ModelSource enum, ModelProvenance struct + impl, and all model-related functions: `resolve_model_alias`, `resolve_model_alias_with_config`, `resolve_repl_model`, `validate_model_syntax`, `config_model_for_current_dir`, `config_alias_for_current_dir`, `max_tokens_for_model`
- [x] 2.2 Add `mod model;` to `main.rs` and re-export via `pub(crate) use model::*;`
- [x] 2.3 Remove original definitions from `main.rs`; add necessary imports to model.rs
- [x] 2.4 Run `cargo build` and `cargo test` — must pass

## 3. Create permission.rs — extract permission prompt logic

- [x] 3.1 Create `src/permission.rs` with CliPermissionPrompter struct, its impl, PermissionPrompter trait impl, and `permission_policy` function (main.rs lines 7442-7508, 8861-8872)
- [x] 3.2 Add `mod permission;` to `main.rs` and re-export
- [x] 3.3 Run `cargo build` and `cargo test` — must pass

## 4. Create progress.rs — extract internal progress reporting

- [x] 4.1 Create `src/progress.rs` with InternalPromptProgressState/Event/Shared/Reporter/Run structs+impls, CliHookProgressReporter, `format_internal_prompt_progress_line`, and `describe_tool_progress` (main.rs lines 7000-7420)
- [x] 4.2 Add `mod progress;` to `main.rs` and re-export
- [x] 4.3 Run `cargo build` and `cargo test` — must pass

## 5. Create doctor.rs — extract diagnostic check system

- [x] 5.1 Create `src/doctor.rs` with DiagnosticLevel, DiagnosticCheck, DoctorReport, `render_doctor_report`, `run_doctor`, and all `check_*_health` functions (main.rs lines 1831-2525)
- [x] 5.2 Add `mod doctor;` to `main.rs` and re-export
- [x] 5.3 Run `cargo build` and `cargo test` — must pass

## 6. Create args.rs — extract all argument parsing

- [ ] 6.1 Create `src/args.rs` with CliAction enum, CliOutputFormat, LocalHelpTopic, `parse_args`, all parse_* helpers, format_unknown_* functions, suggest_*/levenshtein helpers, normalize_* tools, permission_mode helpers, provider_label, filter_tool_specs (main.rs lines 506-1828)
- [ ] 6.2 Add `mod args;` to `main.rs` and re-export
- [ ] 6.3 Run `cargo build` and `cargo test` — must pass

## 7. Create session.rs — extract session management

- [ ] 7.1 Create `src/session.rs` with SessionHandle, ManagedSessionSummary, all session CRUD functions: `sessions_dir`, `current_session_store`, `new_cli_session`, `create_managed_session_handle`, `resolve_session_reference`, `list_managed_sessions`, `latest_managed_session`, `load_session_reference`, `delete_managed_session`, `confirm_session_deletion`, `render_session_list`, `format_session_modified_age`, `write_session_clear_backup`, `session_clear_backup_path` (main.rs lines 3678-3692, 5257-5427)
- [ ] 7.2 Add `mod session;` to `main.rs` and re-export
- [ ] 7.3 Run `cargo build` and `cargo test` — must pass

## 8. Create status.rs — extract status snapshot and git parsing

- [ ] 8.1 Create `src/status.rs` with StatusContext, StatusUsage, GitWorkspaceSummary, `print_status_snapshot`, `status_json_value`, `status_context`, `format_status_report`, `format_sandbox_report`, `print_sandbox_status_snapshot`, `sandbox_json_value`, and all git parsing helpers (main.rs lines 2842-3200, 5462-5839)
- [ ] 8.2 Add `mod status;` to `main.rs` and re-export
- [ ] 8.3 Run `cargo build` and `cargo test` — must pass

## 9. Create formatting.rs — extract tool result formatting and help text

- [ ] 9.1 Create `src/formatting.rs` with all format_tool_* functions, push_output_block, response_to_events, push_prompt_cache_record, prompt_cache_record_to_runtime_event, final_assistant_text, collect_tool_uses/results/cache_events, short_tool_id, extract_tool_path, first_visible_line, truncate_* helpers, and full help text functions (main.rs lines 8184-8730, 8923-9122)
- [ ] 9.2 Add `mod formatting;` to `main.rs` and re-export
- [ ] 9.3 Run `cargo build` and `cargo test` — must pass

## 10. Create config.rs — extract config and memory reporting

- [ ] 10.1 Create `src/config.rs` with `render_config_report`, `render_config_json`, `render_memory_report`, `render_memory_json` (main.rs lines 5962-6143)
- [ ] 10.2 Add `mod config;` to `main.rs` and re-export
- [ ] 10.3 Run `cargo build` and `cargo test` — must pass

## 11. Create diff.rs — extract git diff reporting

- [ ] 11.1 Create `src/diff.rs` with `render_diff_report`, `render_diff_report_for`, `render_diff_json_for`, `run_git_diff_command_in` (main.rs lines 6189-6270)
- [ ] 11.2 Add `mod diff;` to `main.rs` and re-export
- [ ] 11.3 Run `cargo build` and `cargo test` — must pass

## 12. Create export.rs — extract session export

- [ ] 12.1 Create `src/export.rs` with `run_export`, `render_export_text`, `render_session_markdown`, `default_export_filename`, `resolve_export_path`, `summarize_tool_payload_for_markdown` (main.rs lines 6646-6914)
- [ ] 12.2 Add `mod export;` to `main.rs` and re-export
- [ ] 12.3 Run `cargo build` and `cargo test` — must pass

## 13. Create api_client.rs — extract AnthropicRuntimeClient

- [ ] 13.1 Create `src/api_client.rs` with AnthropicRuntimeClient struct, its impl, ApiClient trait impl, `resolve_cli_auth_source`, `resolve_cli_auth_source_for_cwd`, `request_ends_with_tool_result`, `format_user_visible_api_error`, `format_context_window_blocked_error` (main.rs lines 7497-7912)
- [ ] 13.2 Add `mod api_client;` to `main.rs` and re-export
- [ ] 13.3 Run `cargo build` and `cargo test` — must pass

## 14. Create tool_executor.rs — extract CliToolExecutor

- [ ] 14.1 Create `src/tool_executor.rs` with CliToolExecutor struct+impl, ToolExecutor trait impl, ToolSearchRequest/McpToolRequest/ListMcpResourcesRequest/ReadMcpResourceRequest structs, and `convert_messages` (main.rs lines 8731-8922)
- [ ] 14.2 Add `mod tool_executor;` to `main.rs` and re-export
- [ ] 14.3 Run `cargo build` and `cargo test` — must pass

## 15. Create runtime_builder.rs — extract runtime construction

- [ ] 15.1 Create `src/runtime_builder.rs` with RuntimePluginState, RuntimeMcpState, BuiltRuntime struct+impl+Deref+DerefMut+Drop, HookAbortMonitor, all `build_runtime*` functions, `build_plugin_manager`, `resolve_plugin_path`, `runtime_hook_config_from_plugin_hooks`, `build_system_prompt`, `build_runtime_mcp_state`, `mcp_runtime_tool_definition`, `mcp_wrapper_tool_definitions`, `permission_mode_for_mcp_tool`, `mcp_annotation_flag` (main.rs lines 3710-4180, 6924-6995, 7329-7450, 4017-4125)
- [ ] 15.2 Add `mod runtime_builder;` to `main.rs` and re-export
- [ ] 15.3 Run `cargo build` and `cargo test` — must pass

## 16. Create live_cli.rs — extract LiveCli core and REPL loop

- [ ] 16.1 Create `src/live_cli.rs` with LiveCli struct, PromptHistoryEntry, and core impl methods: `new`, `startup_banner`, `prepare_turn_runtime`, `replace_runtime`, `run_turn`, `run_turn_with_output`, `run_prompt_compact/json`, `repl_completion_candidates`, `persist_session`, `record_prompt_history`, `run_internal_prompt_text*`, `reload_runtime_features` (main.rs lines 3694-3710, 4181-4440, 4580-4610, 5240-5260)
- [ ] 16.2 Extract free functions: `run_repl`, `run_resume_command`, `resume_session`, `detect_broad_cwd`, `enforce_broad_cwd_policy`, `run_stale_base_preflight` (main.rs lines 3520-3675, 3597-3678, 3175-3520, 3609-3610)
- [ ] 16.3 Add `mod live_cli;` to `main.rs` and re-export
- [ ] 16.4 Run `cargo build` and `cargo test` — must pass

## 17. Create repl_commands.rs — extract REPL command handlers

- [ ] 17.1 Create `src/repl_commands.rs` with free functions for all REPL command handlers: `handle_repl_command`, `handle_session_command`, `handle_plugins_command`, `set_model`, `set_permissions`, `clear_session`, `compact`, all `print_*` methods, `run_bughunter/ultraplan/teleport/debug_tool_call/commit/pr/issue`, `export_session` (main.rs lines 4440-5256)
- [ ] 17.2 Extract all standalone command functions: `render_repl_help`, `render_resume_usage`, `render_teleport_report`, `render_last_tool_debug_report`, `render_prompt_history_report`, `render_version_report`, `collect_session_prompt_history`, `recent_user_context`, `parse_history_count`, `format_history_timestamp`, `civil_from_days`, `git_output`, `git_status_ok`, `command_exists`, `write_temp_text_file`, `sanitize_generated_message`, `parse_titled_body`, `indent_block`, `validate_no_args`, `init_claude_md`, `normalize_permission_mode`, `run_init`, `init_json_value`, `print_acp_status`, `run_worker_state`, `run_mcp_serve`, `print_bootstrap_plan`, `print_system_prompt`, `print_version`, `version_json_value`, `slash_command_completion_candidates_with_sessions`, `truncate_for_prompt`, `run_export`, `render_export_text`, `render_session_markdown`, `default_export_filename`, `resolve_export_path`, `summarize_tool_payload_for_markdown`
- [ ] 17.3 Extract all format_*_report/notice functions: `format_bughunter_report`, `format_ultraplan_report`, `format_pr_report`, `format_issue_report`, `format_commit_preflight_report`, `format_commit_skipped_report`, `format_auto_compaction_notice`, `format_compact_report`, `format_unknown_slash_command_message`, `format_model_report`, `format_model_switch_report`, `format_permissions_report`, `format_permissions_switch_report`, `format_cost_report`, `format_resume_report`, `render_resume_usage`
- [ ] 17.4 Add `mod repl_commands;` to `main.rs` and re-export
- [ ] 17.5 Run `cargo build` and `cargo test` — must pass

## 18. Slim main.rs and verify

- [ ] 18.1 Remove all moved code from `main.rs`; keep only `main()`, `run()`, `classify_error_kind`, `split_error_hint`, `read_piped_stdin`, `merge_prompt_with_stdin`, `write_mcp_server_fixture`, module declarations, re-exports, and the test module
- [ ] 18.2 Move `#[allow(...)]` directives to individual modules where needed
- [ ] 18.3 Run `cargo build` — must compile cleanly
- [ ] 18.4 Run full test suite: `cargo test` — all tests pass
- [ ] 18.5 Verify `wc -l src/main.rs` ≤ 2000 lines
- [ ] 18.6 Verify no new file exceeds 2000 lines
