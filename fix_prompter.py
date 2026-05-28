import re

with open("rust/crates/rusty-claude-cli/src/permissions/prompter.rs", "r") as f:
    content = f.read()

content = re.sub(r"^fn (parse_permission_mode_arg|permission_mode_from_label|permission_mode_from_resolved|default_permission_mode|config_permission_mode_for_current_dir|permission_policy)\(", r"pub(crate) fn \1(", content, flags=re.M)
content = re.sub(r"^struct CliPermissionPrompter", r"pub(crate) struct CliPermissionPrompter", content, flags=re.M)
content = re.sub(r"fn new\(", r"pub(crate) fn new(", content)

prefix = """use std::env;
use std::io::{self, Write};
use runtime::{PermissionMode, ResolvedPermissionMode, ConfigLoader, PermissionPolicy};
use tools::GlobalToolRegistry;

"""

with open("rust/crates/rusty-claude-cli/src/permissions/prompter.rs", "w") as f:
    f.write(prefix + content)

print("done")
