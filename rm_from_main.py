import re

MAIN_FILE = "rust/crates/rusty-claude-cli/src/main.rs"

with open(MAIN_FILE, 'r') as f:
    content = f.read()

# remove AnthropicRuntimeClient from main.rs
content = re.sub(r'struct AnthropicRuntimeClient \{.*?\n\}\n\nimpl AnthropicRuntimeClient \{.*?\n\}\n\nimpl runtime::Client for AnthropicRuntimeClient \{.*?\n\}', '', content, flags=re.DOTALL | re.MULTILINE)

with open(MAIN_FILE, 'w') as f:
    f.write(content)

print("Removed from main.rs")
