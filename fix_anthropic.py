import re

with open('anthropic_full.rs', 'r') as f:
    content = f.read()

content = re.sub(r'^struct ', r'pub(crate) struct ', content, flags=re.MULTILINE)
content = re.sub(r'^(\s+)(async )?fn ', r'\1pub(crate) \2fn ', content, flags=re.MULTILINE)

# except for trait methods
content = re.sub(r'pub\(crate\) fn name\(', r'fn name(', content)
content = re.sub(r'pub\(crate\) async fn process_stream\(', r'async fn process_stream(', content)
content = re.sub(r'pub\(crate\) fn session_id\(', r'fn session_id(', content)
content = re.sub(r'pub\(crate\) fn model\(', r'fn model(', content)
content = re.sub(r'pub\(crate\) fn enable_tools\(', r'fn enable_tools(', content)

with open('anthropic_full.rs', 'w') as f:
    f.write(content)

with open('rust/crates/rusty-claude-cli/src/execution/client.rs', 'a') as f:
    f.write("\n" + content + "\n")
