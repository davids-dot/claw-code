## Context

The `rusty-claude-cli/src/main.rs` file is currently a monolithic file containing over 9,200 lines of code. It handles everything from CLI argument parsing and session management to output rendering and error formatting. While we have successfully extracted the ~4,300 lines of test code into `main_tests.rs`, the remaining codebase is still too large for a single file, making maintenance, navigation, and parallel development difficult.

## Goals / Non-Goals

**Goals:**
- Decompose the `main.rs` file into a modular structure aligned with Rust community best practices.
- Improve code readability, maintainability, and compilation times.
- Ensure 100% of the existing test suite continues to pass without modification to the core logic.

**Non-Goals:**
- Do not add any new features or change existing business logic.
- Do not refactor external dependencies or change the CLI's public-facing behavior.
- Do not rewrite the logic inside functions—this is strictly a structural reorganization.

## Decisions

1. **Incremental "Strangler Fig" Extraction Strategy:**
   - *Rationale:* Moving 9,200 lines at once is highly risky and guarantees import nightmares. Extracting one logical domain at a time (e.g., pure data structures first, then independent utility functions) minimizes blast radius and makes resolving `use` statements manageable.
2. **Module Taxonomy:**
   - `models.rs` / `types.rs`: Will house pure data structures like `ModelSource`, `ModelProvenance`, `SessionLifecycleSummary`, and API request/response schemas. These are the leaves of the dependency tree.
   - `error.rs`: Will contain the global error enums, `classify_error_kind`, and `split_error_hint`.
   - `render.rs` / `ui.rs`: Will handle output formatting, markdown rendering, and CLI visual components.
   - `repl.rs`: Will encapsulate the `LiveCli` struct and the `run_repl` interactive loop.
   - `cli/`: (Optional) Further split command handlers (e.g., `cli/resume.rs`, `cli/config.rs`) if the command parsing logic remains too large.
3. **Keep `main.rs` as a Thin Entrypoint:**
   - *Rationale:* `main.rs` should only define the top-level `mod` declarations, parse CLI arguments, invoke the appropriate runner from the submodules, and handle the final `std::process::exit` codes.

## Risks / Trade-offs

- **Risk: Import Resolution Chaos**
  - *Trade-off:* Moving structs to new files means `use crate::models::*` will need to be added across many places.
  - *Mitigation:* Rely heavily on `cargo check` and the rust-analyzer LSP. We will extract the "leaves" of the dependency graph (pure models/errors) first before moving the "trunks" (the REPL loop).
- **Risk: Test Suite Breakage**
  - *Trade-off:* `main_tests.rs` currently relies heavily on `use crate::*` to import private functions from `main.rs`. Moving them to submodules might require making some previously private functions `pub(crate)`.
  - *Mitigation:* Expose extracted functions as `pub(crate)` within their new modules so that `main.rs` and `main_tests.rs` can still access them.