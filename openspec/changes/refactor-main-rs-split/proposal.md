## Why

`rusty-claude-cli/src/main.rs` is 13,292 lines / 473KB — a single-file God Module containing all CLI logic except three thin modules (init, input, render). This makes it extremely difficult to:

- **Navigate**: finding a specific function requires scrolling through thousands of lines
- **Reason about**: no module-level boundaries document what belongs where
- **Test**: all functions share file-private visibility, forcing brittle integration tests
- **Maintain**: any change risks unintended side effects across unrelated subsystems

The project rule requires any file ≤ 2000 lines. This change brings `main.rs` under that limit by splitting its contents into focused, responsibility-driven modules.

## What Changes

**Phase 1 — Pure vertical split (zero behavior change):**

Move functions, structs, and impl blocks from `main.rs` into 15+ new files by thematic responsibility. Each new file declares a clear boundary. `main.rs` retains only the entry point (`main`, `run`) and the `CliAction` enum dispatch. All other code moves to new modules with `pub(crate)` visibility where cross-module access is needed.

- **`constants.rs`** — all `const` definitions and type aliases
- **`model.rs`** — `ModelSource`, `ModelProvenance`, model alias resolution
- **`args.rs`** — `CliAction` enum, `CliOutputFormat`, `parse_args()` + 50+ parse helpers
- **`doctor.rs`** — `DiagnosticLevel`, `DiagnosticCheck`, `DoctorReport`, health checks
- **`session.rs`** — `SessionHandle`, `ManagedSessionSummary`, session CRUD
- **`status.rs`** — `StatusContext`, `GitWorkspaceSummary`, status/sandbox snapshots
- **`permission.rs`** — `CliPermissionPrompter`, `PermissionPrompter` impl
- **`progress.rs`** — `InternalPromptProgressReporter/Run`, `HookProgressReporter` impl
- **`api_client.rs`** — `AnthropicRuntimeClient`, `ApiClient` impl
- **`tool_executor.rs`** — `CliToolExecutor`, `ToolExecutor` impl
- **`runtime_builder.rs`** — `BuiltRuntime`, `RuntimePluginState`, `RuntimeMcpState`, build functions
- **`formatting.rs`** — all `format_tool_*`, `push_output_block`, `response_to_events`, help text
- **`config.rs`** — config and memory report rendering
- **`diff.rs`** — git diff report rendering
- **`export.rs`** — session export (text/markdown) functions
- **`live_cli.rs`** — `LiveCli` struct + core methods (new, run_turn, persist_session, etc.)
- **`repl_commands.rs`** — `LiveCli` command handlers (`handle_repl_command`, `/status`, `/model`, etc.) and all standalone command functions

Existing files (`init.rs`, `input.rs`, `render.rs`) remain unchanged.

**Phase 2 — Architectural refinement (optional, future):**

- Extract `CliApiClient` / `CliOutputPort` / `SessionStore` traits
- Consider Command Handler pattern for REPL commands

## Capabilities

### New Capabilities

This is a **pure refactoring** with no new user-facing capabilities. The module boundaries correspond to internal architectural slices:

- `cli-args-parsing`: CLI argument parsing, validation, and suggestion system
- `cli-session-management`: session lifecycle (create, list, switch, delete, fork)
- `cli-runtime-construction`: runtime, plugin, and MCP server bootstrapping
- `cli-output-formatting`: all terminal output formatting (tool results, reports, help)
- `cli-repl-interaction`: REPL loop and slash command dispatch

### Modified Capabilities

None. No spec-level behavior changes — this is purely a code organization refactor.

## Impact

| Area | Impact |
|------|--------|
| **`rusty-claude-cli/src/`** | 3 files → ~20 files. Existing modules (init, input, render) untouched. |
| **`main.rs`** | 13,292 → ~800 lines (only main(), run(), CliAction, re-exports) |
| **Visibility** | Many private `fn` become `pub(crate)` for cross-module access |
| **Testing** | Existing tests remain valid; some `use super::*` imports need updating |
| **Compilation** | No dependency changes at the Cargo workspace level |
| **Behavior** | Zero. Pure move-and-rename refactoring. |
