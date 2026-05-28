import os
import re

files = [
    'rust/crates/rusty-claude-cli/src/execution/client.rs',
    'rust/crates/rusty-claude-cli/src/execution/stream.rs',
    'rust/crates/rusty-claude-cli/src/execution/executor.rs'
]

for file in files:
    with open(file, 'r') as f:
        content = f.read()

    # Change fn to pub(crate) fn
    content = re.sub(r'^(async )?fn ', r'pub(crate) \1fn ', content, flags=re.MULTILINE)
    content = re.sub(r'pub\(crate\) pub\(crate\) ', r'pub(crate) ', content)
    
    # Inside impl blocks, we also want pub(crate) fn
    content = re.sub(r'^(\s+)(async )?fn ', r'\1pub(crate) \2fn ', content, flags=re.MULTILINE)
    
    # Structs
    content = re.sub(r'^struct ', r'pub(crate) struct ', content, flags=re.MULTILINE)
    
    # Struct fields
    # Match structs block and add pub(crate) to fields
    def repl_struct(m):
        inner = m.group(2)
        # replace field definitions: "name: type,"
        inner = re.sub(r'^(\s+)([a-zA-Z0-9_]+)\s*:\s*', r'\1pub(crate) \2: ', inner, flags=re.MULTILINE)
        return m.group(1) + inner + m.group(3)
        
    content = re.sub(r'(pub\(crate\) struct [a-zA-Z0-9_]+ \{)(.*?)(\})', repl_struct, content, flags=re.DOTALL)
    
    with open(file, 'w') as f:
        f.write(content)

print("Visibility updated.")
