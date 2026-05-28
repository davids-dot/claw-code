import re

MAIN_FILE = "rust/crates/rusty-claude-cli/src/main.rs"

with open(MAIN_FILE, 'r') as f:
    content = f.read()

def extract_and_remove(pattern, content):
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if match:
        extracted = match.group(0)
        content = content.replace(extracted, "")
        return extracted, content
    return None, content

executor_pattern = r'struct CliToolExecutor \{.*?\n\}\n\nimpl CliToolExecutor \{.*?\n\}\n\nimpl ToolExecutor for CliToolExecutor \{.*?\n\}'
executor_code, content = extract_and_remove(executor_pattern, content)

with open(MAIN_FILE, 'w') as f:
    f.write(content)

with open('executor_extracted.rs', 'w') as f:
    if executor_code:
        f.write(executor_code + "\n\n")

print("Executor extraction complete")
