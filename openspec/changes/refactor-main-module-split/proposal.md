## Why

The `rusty-claude-cli/src/main.rs` file has grown to over 6200 lines, violating the Single Responsibility Principle and becoming a "God File". This high coupling makes the code difficult to read, maintain, and test. It also slows down compilation because all logic resides in a single file. Refactoring is necessary now to improve maintainability, reduce developer cognitive load, and enable easier addition of future features.

## What Changes

- Split `main.rs` into smaller, focused modules based on Domain-Driven Design (DDD) boundaries.
- Create `cli` module for command-line argument parsing, prompt actions, and spelling suggestions.
- Create `config` module for model resolution and configuration management.
- Create `execution` module for the core tool execution engine (`CliToolExecutor`) and API client runtime interactions (`AnthropicRuntimeClient`).
- Create `permissions` module for security, policy enforcement, and permission prompting (`CliPermissionPrompter`).
- Create `diagnostics` module for diagnostic checks and error states.
- Create `ui` module for terminal rendering, hooks, and progress tracking.
- Shrink `main.rs` so it only contains the entry point (`main()` function), global initialization, and top-level routing.

## Capabilities

### New Capabilities
- `main-module-refactoring`: Refactoring of the `main.rs` file into modular components (`cli`, `config`, `execution`, `permissions`, `diagnostics`, `ui`) without altering the external CLI behavior.

### Modified Capabilities
- (None - This is a pure refactoring change; no external requirements or behaviors are changing.)

## Impact

- `rusty-claude-cli/src/main.rs` will be heavily reduced in size.
- Numerous new files and folders will be created under `rusty-claude-cli/src/` (e.g., `src/cli/`, `src/execution/`, etc.).
- Internal module imports (`use` statements) across the entire `rusty-claude-cli` crate will be updated to reflect the new structure.
- Incremental compilation speed should improve.
- Unit tests currently residing in `main.rs` or `main_tests.rs` might need to be relocated to their respective new modules.
