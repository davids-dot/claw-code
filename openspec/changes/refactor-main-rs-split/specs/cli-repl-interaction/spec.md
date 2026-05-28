## ADDED Requirements

### Requirement: LiveCli core methods shall be in `live_cli.rs`

The `LiveCli` struct definition and its core lifecycle methods SHALL be in
`live_cli.rs`. Core methods include `new`, `startup_banner`, `prepare_turn_runtime`,
`replace_runtime`, `run_turn`, `run_turn_with_output`, `run_prompt_compact`,
`run_prompt_compact_json`, `run_prompt_json`, `run_internal_prompt_text_with_progress`,
`run_internal_prompt_text`, `persist_session`, `record_prompt_history`,
`repl_completion_candidates`, and `reload_runtime_features`.

#### Scenario: live_cli.rs contains LiveCli struct

- **WHEN** the refactoring is complete
- **THEN** `live_cli.rs` defines `LiveCli` struct and contains all core lifecycle
  methods listed above

#### Scenario: run_repl is in live_cli.rs

- **WHEN** the refactoring is complete
- **THEN** `live_cli.rs` contains `run_repl`, `run_resume_command`,
  and `resume_session` as free functions

### Requirement: REPL command handlers shall be free functions in `repl_commands.rs`

All `handle_repl_command`, `handle_session_command`, `handle_plugins_command`,
`set_model`, `set_permissions`, `clear_session`, `compact`, `print_*` methods,
`run_bughunter`, `run_ultraplan`, `run_teleport`, `run_debug_tool_call`,
`run_commit`, `run_pr`, `run_issue`, `export_session`, and all format_*_report
helpers SHALL be free functions in `repl_commands.rs` that accept
`&mut LiveCli` or `&LiveCli` as their first parameter.

#### Scenario: handle_repl_command is a free function

- **WHEN** the refactoring is complete
- **THEN** `repl_commands.rs` defines `pub(crate) fn handle_repl_command(
  cli: &mut LiveCli, command: SlashCommand) -> Result<bool, Box<dyn std::error::Error>>`

#### Scenario: All /slash commands are handled in repl_commands.rs

- **WHEN** user types `/status`, `/model`, `/session`, `/config`, `/diff`
- **THEN** the dispatch function in `repl_commands.rs` handles it via a
  free function that takes `cli: &mut LiveCli`

### Requirement: REPL command dispatch shall not contain runtime construction

No REPL command handler SHALL directly call `build_runtime`, `ConversationRuntime::new`,
or `ApiProviderClient` constructors. Any runtime replacement SHALL go through
`LiveCli::replace_runtime`.

#### Scenario: Runtime is only replaced through LiveCli::replace_runtime

- **WHEN** a REPL command changes model or permissions
- **THEN** it calls `cli.replace_runtime(build_runtime(...))` from `runtime_builder.rs`
- **THEN** it does NOT directly construct `ConversationRuntime` or manage plugin
  lifecycle

### Requirement: Doctor diagnostics shall be in `doctor.rs`

`DiagnosticLevel`, `DiagnosticCheck`, `DoctorReport`, `run_doctor`, all
`check_*_health` functions, and `render_diagnostic_check` SHALL be in `doctor.rs`.

#### Scenario: doctor.rs exists

- **WHEN** the refactoring is complete
- **THEN** `doctor.rs` contains the full diagnostic check system

### Requirement: Model resolution shall be in `model.rs`

`ModelSource`, `ModelProvenance`, `resolve_model_alias`,
`resolve_model_alias_with_config`, `resolve_repl_model`, `validate_model_syntax`,
`config_model_for_current_dir`, `config_alias_for_current_dir`, and
`max_tokens_for_model` SHALL be in `model.rs`.

#### Scenario: model.rs exists

- **WHEN** the refactoring is complete
- **THEN** `model.rs` contains all model-related types and functions

### Requirement: Status snapshot shall be in `status.rs`

`StatusContext`, `StatusUsage`, `GitWorkspaceSummary`, `print_status_snapshot`,
`status_json_value`, `status_context`, `format_status_report`,
`format_sandbox_report`, `print_sandbox_status_snapshot`, `sandbox_json_value`,
and all git status parsing helpers SHALL be in `status.rs`.

#### Scenario: status.rs exists

- **WHEN** the refactoring is complete
- **THEN** `status.rs` contains all status and git parsing functions
