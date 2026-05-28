sed -i '' '1i\
use crate::{CliOutputFormat, OFFICIAL_REPO_SLUG, DEPRECATED_INSTALL_COMMAND, OFFICIAL_REPO_URL, resolve_cli_auth_source_for_cwd};\
use tools::{mvp_tool_specs, execute_tool};\
use runtime::{McpTool, McpServerSpec, McpServer, load_oauth_credentials};\
' rust/crates/rusty-claude-cli/src/diagnostics/mod.rs

sed -i '' 's/fn run_doctor(/pub(crate) fn run_doctor(/g' rust/crates/rusty-claude-cli/src/diagnostics/mod.rs
sed -i '' 's/fn run_worker_state(/pub(crate) fn run_worker_state(/g' rust/crates/rusty-claude-cli/src/diagnostics/mod.rs
sed -i '' 's/fn run_mcp_serve(/pub(crate) fn run_mcp_serve(/g' rust/crates/rusty-claude-cli/src/diagnostics/mod.rs
