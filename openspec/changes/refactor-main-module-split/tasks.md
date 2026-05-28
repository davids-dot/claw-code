## 1. Initial Setup & Leaf Node Extraction

- [x] 1.1 Create the directory structure: `src/cli`, `src/config`, `src/execution`, `src/permissions`, `src/diagnostics`, `src/ui`.
- [x] 1.2 Create `mod.rs` files in each new directory to expose them to `main.rs`.
- [x] 1.3 Extract `DiagnosticCheck` and `DiagnosticLevel` related code into `src/diagnostics/mod.rs`. Update visibility to `pub(crate)` and fix imports in `main.rs`.
- [x] 1.4 Extract model validation and alias resolution (`resolve_model_alias`, `validate_model_syntax`) into `src/config/models.rs`. Fix imports in `main.rs`.
- [x] 1.5 Extract spelling and suggestion logic (`suggest_slash_commands`, `levenshtein_distance`) into `src/cli/suggestions.rs`. Fix imports in `main.rs`.
- [x] 1.6 Verify compilation (`cargo check`) and format (`cargo fmt --all`) after leaf node extraction.

## 2. Intermediate Node Extraction

- [x] 2.1 Extract terminal UI progress and hooks (`CliHookProgressReporter`, `describe_tool_progress`) into `src/ui/progress.rs`. Fix imports.
- [x] 2.2 Extract permission policies and prompting (`CliPermissionPrompter`, `permission_policy`) into `src/permissions/prompter.rs`. Fix imports.
- [x] 2.3 Move any relevant unit tests from the bottom of `main.rs` to their new respective modules (`diagnostics`, `config`, `cli/suggestions`, `ui`, `permissions`).
- [x] 2.4 Verify compilation (`cargo check`) and run tests (`cargo test`).

## 3. Core Engine Extraction

- [x] 3.1 Extract API stream consumption logic (`consume_stream`, `response_to_events`) into `src/execution/stream.rs`. Fix imports.
- [x] 3.2 Extract the API client wrapper (`AnthropicRuntimeClient`, `build_runtime`) into `src/execution/client.rs`. Fix imports.
- [x] 3.3 Extract the tool execution engine (`CliToolExecutor`, `execute_runtime_tool`) into `src/execution/executor.rs`. This may require careful handling of circular dependencies with `ui` and `permissions`.
- [x] 3.4 Move relevant tests from `main.rs` to `src/execution/`.
- [x] 3.5 Verify compilation (`cargo check`) and tests (`cargo test`).

## 4. CLI Parser Extraction

- [ ] 4.1 Extract command line argument parsing (`parse`, `CliAction`, `parse_acp_args`, etc.) into `src/cli/parser.rs`. Fix imports.
- [ ] 4.2 Move CLI parsing tests to `src/cli/parser.rs`.
- [ ] 4.3 Verify compilation (`cargo check`) and tests (`cargo test`).

## 5. Final Cleanup

- [ ] 5.1 Clean up `src/main.rs`, removing unused imports and dead code. Ensure it only acts as the main entry point and module router.
- [ ] 5.2 Run a final `cargo check`, `cargo test`, and `cargo fmt --all` to guarantee no regressions.
