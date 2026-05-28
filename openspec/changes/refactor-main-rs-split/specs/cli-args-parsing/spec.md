## ADDED Requirements

### Requirement: CLI argument parsing shall be isolated in `args.rs`

All CLI argument parsing logic SHALL be contained in a single `args.rs` module.
This includes `parse_args()`, all `parse_*` helper functions, the `CliAction` enum,
`CliOutputFormat`, `LocalHelpTopic`, and all suggestion/error-reporting helpers
(Levenshtein distance, ranked suggestions, unknown-command formatting).

#### Scenario: All parse functions are in args.rs

- **WHEN** the refactoring is complete
- **THEN** `args.rs` contains `parse_args`, `parse_export_args`, `parse_resume_args`,
  `parse_system_prompt_args`, `parse_dump_manifests_args`, `parse_local_help_action`,
  `parse_direct_slash_cli_action`, `parse_acp_args`, and all format_unknown_* /
  suggest_* / levenshtein_distance helpers

#### Scenario: args.rs does not contain runtime logic

- **WHEN** any function in `args.rs` is reviewed
- **THEN** it SHALL only perform argument parsing and validation, never executing
  business logic (no session CRUD, no runtime construction, no API calls)

### Requirement: `args.rs` shall expose `pub(crate)` parse functions

Argument parsing functions SHALL be `pub(crate)` so they are accessible from
`main.rs` for dispatch but not from outside the crate.

#### Scenario: main.rs can call parse_args

- **WHEN** `main.rs` calls `args::parse_args(&args)`
- **THEN** it returns a `Result<CliAction, String>`
- **THEN** no other crate module can call parse functions (no `pub` visibility)

### Requirement: Suggestion system shall remain in args.rs

All fuzzy-matching and suggestion utilities (`suggest_slash_commands`,
`suggest_closest_term`, `ranked_suggestions`, `levenshtein_distance`,
`common_prefix_len`) SHALL reside in `args.rs`.

#### Scenario: Typo suggestions work after refactoring

- **WHEN** user types an unknown command
- **THEN** `format_unknown_slash_command` produces a suggestion with the same
  formatting as before refactoring
