import os

files = [
    'rust/crates/rusty-claude-cli/src/execution/client.rs',
    'rust/crates/rusty-claude-cli/src/execution/stream.rs',
    'rust/crates/rusty-claude-cli/src/execution/executor.rs'
]

imports = """
use crate::*;
use api::*;
use runtime::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;
"""

for file in files:
    with open(file, 'r') as f:
        content = f.read()
    with open(file, 'w') as f:
        f.write(imports + "\n" + content)

main_file = "rust/crates/rusty-claude-cli/src/main.rs"
with open(main_file, 'r') as f:
    main_content = f.read()

main_imports = """
use crate::execution::client::*;
use crate::execution::stream::*;
use crate::execution::executor::*;
"""
# inject right after mod declarations
main_content = main_content.replace("pub(crate) mod ui;\n", "pub(crate) mod ui;\n" + main_imports)

with open(main_file, 'w') as f:
    f.write(main_content)

print("Imports added.")
