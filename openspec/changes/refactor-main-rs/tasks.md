## 1. Extract Data Models

- [x] 1.1 Create `src/models.rs` file
- [x] 1.2 Move pure structs and enums (`ModelSource`, `ModelProvenance`, `SessionLifecycleSummary`, etc.) from `main.rs` to `models.rs`
- [x] 1.3 Add `pub(crate)` visibility to the moved items
- [x] 1.4 Update `main.rs` and `main_tests.rs` to `use crate::models::*`
- [x] 1.5 Run `cargo check` and `cargo test` to ensure successful compilation and tests

## 2. Extract Error Handling

- [x] 2.1 Create `src/error.rs` file
- [x] 2.2 Move `classify_error_kind`, `split_error_hint`, and global error enums to `error.rs`
- [x] 2.3 Update imports across `main.rs`, `main_tests.rs`, and `models.rs`
- [x] 2.4 Verify with `cargo check` and `cargo test`

## 3. Extract Rendering & UI Logic

- [x] 3.1 Create `src/render.rs` file
- [x] 3.2 Move output formatting, Markdown rendering logic, and UI reports (e.g., `format_model_report`, `push_output_block`)
- [x] 3.3 Re-wire imports and resolve any dependency cycles between `render.rs` and `models.rs`
- [x] 3.4 Verify with `cargo check` and `cargo test`

## 4. Extract REPL & Core Engine

- [x] 4.1 Create `src/repl.rs` file
- [x] 4.2 Move `LiveCli` struct and `run_repl` loop
- [x] 4.3 Move command parsing and execution logic
- [x] 4.4 Finalize import paths across all newly extracted modules
- [x] 4.5 Run full `cargo test --workspace` and `cargo fmt --all`

## 5. Cleanup `main.rs`

- [x] 5.1 Ensure `main.rs` only contains module declarations (`mod models; mod error;`, etc.), CLI argument definitions, and the `fn main()` entrypoint
- [x] 5.2 Remove unused imports in `main.rs`
- [x] 5.3 Final verification of the `rusty-claude-cli` build