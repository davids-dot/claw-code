## ADDED Requirements

### Requirement: Tool result formatting shall be in `formatting.rs`

All `format_tool_*` functions, `response_to_events`, `push_output_block`,
`push_prompt_cache_record`, and related helpers SHALL be in `formatting.rs`.

#### Scenario: format_tool_call_start is in formatting.rs

- **WHEN** the refactoring is complete
- **THEN** `formatting.rs` contains `format_tool_call_start`, `format_tool_result`,
  `format_bash_call`, `format_bash_result`, `format_read_result`,
  `format_write_result`, `format_edit_result`, `format_glob_result`,
  `format_grep_result`, `format_generic_tool_result`, `format_search_start`,
  `format_patch_preview`, `format_structured_patch_preview`

#### Scenario: stream helpers are in formatting.rs

- **WHEN** the refactoring is complete
- **THEN** `formatting.rs` contains `push_output_block`, `response_to_events`,
  `push_prompt_cache_record`, `prompt_cache_record_to_runtime_event`,
  `final_assistant_text`, `collect_tool_uses`, `collect_tool_results`,
  `collect_prompt_cache_events`, `short_tool_id`

### Requirement: Help text rendering shall be in `formatting.rs`

`print_help_to`, `print_help`, `render_help_topic`, and `print_help_topic`
SHALL be in `formatting.rs`.

#### Scenario: print_help_to is in formatting.rs

- **WHEN** the refactoring is complete
- **THEN** `formatting.rs` contains `print_help_to` and the full help text output

### Requirement: Config, diff, and export rendering shall be in dedicated modules

Config report rendering SHALL be in `config.rs`. Git diff rendering SHALL be
in `diff.rs`. Session export rendering SHALL be in `export.rs`.

#### Scenario: config.rs exists

- **WHEN** the refactoring is complete
- **THEN** `config.rs` contains `render_config_report`, `render_config_json`,
  `render_memory_report`, `render_memory_json`

#### Scenario: diff.rs exists

- **WHEN** the refactoring is complete
- **THEN** `diff.rs` contains `render_diff_report`, `render_diff_report_for`,
  `render_diff_json_for`, `run_git_diff_command_in`

#### Scenario: export.rs exists

- **WHEN** the refactoring is complete
- **THEN** `export.rs` contains `run_export`, `render_export_text`,
  `render_session_markdown`, `default_export_filename`, `resolve_export_path`,
  `summarize_tool_payload_for_markdown`

### Requirement: Internal progress reporting shall be in `progress.rs`

`InternalPromptProgressState`, `InternalPromptProgressEvent`,
`InternalPromptProgressShared`, `InternalPromptProgressReporter`,
`InternalPromptProgressRun`, `CliHookProgressReporter`,
`format_internal_prompt_progress_line`, and `describe_tool_progress`
SHALL be in `progress.rs`.

#### Scenario: progress.rs exists

- **WHEN** the refactoring is complete
- **THEN** `progress.rs` contains all internal progress reporting types and impls
