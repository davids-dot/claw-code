## Why

The `rust/crates/rusty-claude-cli/src/main.rs` file has grown excessively large (currently over 9,200 lines, originally ~13,600 lines before extracting tests). It acts as a "God Object" containing structural models, core engine logic, error handling, output formatting, and session management. This monolithic structure makes the codebase difficult to navigate, maintain, and understand, significantly increasing the cognitive load for developers. Refactoring it into cohesive, single-responsibility modules is necessary to align with Rust community best practices.

## What Changes

- **Extract Data Models**: Move pure data structures and enums (e.g., `ModelSource`, `ModelProvenance`, `SessionLifecycleSummary`) into a dedicated `models.rs` or `types.rs` module.
- **Extract Core Engine**: Move the REPL and core execution logic (e.g., `LiveCli`, `run_repl`) into a new `repl.rs` module.
- **Extract Error Handling**: Move error classification and hint formatting logic (e.g., `classify_error_kind`, `split_error_hint`) into an `error.rs` module.
- **Extract Formatting/UI**: Move output rendering and formatting functions into a `render.rs` or `ui.rs` module.
- **Thin Entrypoint**: Reduce `main.rs` to a thin entry point responsible only for CLI argument parsing, top-level initialization, and command routing.

## Capabilities

### New Capabilities
- None. This is a pure structural refactoring with no new user-facing capabilities.

### Modified Capabilities
- None. Existing behaviors and requirements remain exactly the same.

## Impact

- **Codebase Structure**: `rusty-claude-cli/src/` will contain several new cohesive modules (`models.rs`, `repl.rs`, `error.rs`, etc.).
- **Maintainability**: Vastly improved readability and targeted compilation/testing for future changes.
- **External APIs**: No changes to external APIs, CLI arguments, or dependencies. All changes are strictly internal structural reorganizations.