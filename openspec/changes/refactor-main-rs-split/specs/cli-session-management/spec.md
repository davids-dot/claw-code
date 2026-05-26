## ADDED Requirements

### Requirement: Session management shall be isolated in `session.rs`

All session lifecycle operations (create, resolve, list, delete, search) SHALL
be contained in a single `session.rs` module. This includes `SessionHandle`,
`ManagedSessionSummary`, all session CRUD functions, and session rendering helpers.

#### Scenario: All session CRUD is in session.rs

- **WHEN** the refactoring is complete
- **THEN** `session.rs` contains `sessions_dir`, `current_session_store`,
  `new_cli_session`, `create_managed_session_handle`, `resolve_session_reference`,
  `resolve_managed_session_path`, `list_managed_sessions`, `latest_managed_session`,
  `load_session_reference`, `delete_managed_session`, `confirm_session_deletion`,
  `render_session_list`, `format_session_modified_age`,
  `write_session_clear_backup`, `session_clear_backup_path`

#### Scenario: session.rs does not contain runtime construction

- **WHEN** any function in `session.rs` is reviewed
- **THEN** it SHALL NOT call `build_runtime` or `ConversationRuntime::new`
  — session functions only read/write session files, not construct runtimes

### Requirement: Session types shall be `pub(crate)`

`SessionHandle` and `ManagedSessionSummary` SHALL be `pub(crate)` structs
accessible from `live_cli.rs`, `status.rs`, and `export.rs`.

#### Scenario: LiveCli uses SessionHandle from session.rs

- **WHEN** `live_cli.rs` constructs a `SessionHandle`
- **THEN** it imports it from `crate::session::SessionHandle`
- **THEN** the struct fields are accessible within the crate

### Requirement: Session store path resolution shall be centralized

`sessions_dir()` and `current_session_store()` SHALL be the single source of
truth for where session files are stored on disk.

#### Scenario: All session file paths go through session.rs

- **WHEN** any module needs to know the session directory path
- **THEN** it calls `sessions_dir()` from `session.rs`
- **THEN** no other module duplicates path resolution logic
