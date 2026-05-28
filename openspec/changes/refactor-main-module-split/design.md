## Context

The `rusty-claude-cli` application currently suffers from having a massive entry point. The `src/main.rs` file is over 6,200 lines long, combining CLI argument parsing, configuration loading, API interaction, terminal UI rendering, tool execution, and permission prompting. This violates the Single Responsibility Principle, making the codebase difficult to navigate, test, and maintain. Compiling a single massive file also creates a bottleneck for the Rust compiler.

## Goals / Non-Goals

**Goals:**
- Break down `src/main.rs` into smaller, logically grouped modules following Domain-Driven Design (DDD).
- Establish clear dependency boundaries between these new modules.
- Maintain 100% backward compatibility with the existing CLI behavior.
- Ensure all existing tests pass (`cargo test`).
- Improve incremental compilation times.

**Non-Goals:**
- Introducing any new CLI features, commands, or behaviors.
- Rewriting the internal logic of functions (beyond fixing visibility `pub(crate)` and imports).
- Modifying the underlying `api`, `runtime`, or `tools` crates.

## Decisions

### 1. Target Directory Structure
We will adopt the following module structure:
- `src/main.rs`: Stripped down to the bare minimum. Contains `fn main()`, top-level `mod` declarations, and the root routing logic.
- `src/cli/`: Handles command-line input.
  - `parser.rs`: `parse()`, `CliAction`, `parse_acp_args()`, etc.
  - `suggestions.rs`: Spell checking and command suggestions (`suggest_slash_commands`, `levenshtein_distance`).
- `src/config/`: Configuration and model validation.
  - `models.rs`: `resolve_model_alias()`, `validate_model_syntax()`, etc.
- `src/execution/`: Core engine logic.
  - `executor.rs`: `CliToolExecutor`, `execute_runtime_tool()`.
  - `client.rs`: `AnthropicRuntimeClient`, `build_runtime()`.
  - `stream.rs`: API streaming responses (`consume_stream()`, `response_to_events()`).
- `src/permissions/`: Security policies.
  - `prompter.rs`: `CliPermissionPrompter`, `permission_policy()`.
- `src/diagnostics/`: System health checks.
  - `mod.rs`: `DiagnosticCheck`, `DiagnosticLevel`.
- `src/ui/`: Terminal interactions.
  - `progress.rs`: `CliHookProgressReporter`, `describe_tool_progress()`.

*Rationale:* This structure separates concerns naturally. The `cli` handles user input, `config` manages state, `execution` orchestrates the logic, and `ui` handles the output.

### 2. Incremental Extraction Strategy
Instead of moving all code in one massive cut-and-paste operation, the refactoring will proceed in topological order from the leaves to the root:
1. Extract leaf nodes with few dependencies (e.g., `diagnostics`, `config/models.rs`, `cli/suggestions.rs`).
2. Extract intermediate nodes (e.g., `permissions`, `ui/progress.rs`).
3. Extract core components (`execution`).
4. Extract the top-level parser (`cli/parser.rs`).
5. Update `main.rs` to route to these modules.

*Rationale:* Attempting to split everything simultaneously often leads to overwhelming compiler errors regarding missing imports or circular dependencies. Step-by-step extraction ensures that `cargo check` can validate each step.

### 3. Visibility Changes
Moved structures and functions that were previously private within `main.rs` will be updated to `pub(crate)` so they can be accessed across the new module boundaries within the crate, without exposing them publicly outside the crate.

## Risks / Trade-offs

- **Risk: Circular Dependencies.** Code currently co-located in `main.rs` might have hidden cyclic dependencies that will become apparent when split into separate files.
  - *Mitigation:* Ensure a strict dependency graph: `cli` depends on `execution` and `config`; `execution` depends on `permissions`, `ui`, and `config`. If cycles are found, we may need to introduce shared generic interfaces or move shared types to a common `types.rs` or `models.rs` module.
- **Risk: Broken Unit Tests.** Tests residing at the bottom of `main.rs` might break when functions are moved.
  - *Mitigation:* Move unit tests alongside the code they test into their respective new modules. Verify with `cargo test` after each module extraction.
- **Risk: Extensive Import Updates.** Every file will need its `use` statements heavily updated.
  - *Mitigation:* Rely on the Rust compiler and IDE diagnostics to find and fix missing imports iteratively.
