import re

STREAM_FILE = "rust/crates/rusty-claude-cli/src/execution/stream.rs"
CLIENT_FILE = "rust/crates/rusty-claude-cli/src/execution/client.rs"
EXECUTOR_FILE = "rust/crates/rusty-claude-cli/src/execution/executor.rs"

with open(STREAM_FILE, 'r') as f:
    stream_content = f.read()

# Extract consume_stream
consume_stream_pattern = r'    /// Consume a single streaming response.*?\}\n\n'
match = re.search(consume_stream_pattern, stream_content, re.DOTALL | re.MULTILINE)
if match:
    consume_stream_code = match.group(0)
    stream_content = stream_content.replace(consume_stream_code, "")
    
    with open(STREAM_FILE, 'w') as f:
        f.write(stream_content)
        
    with open(CLIENT_FILE, 'r') as f:
        client_content = f.read()
        
    # insert inside impl AnthropicRuntimeClient
    client_content = client_content.replace("impl AnthropicRuntimeClient {", "impl AnthropicRuntimeClient {\n" + consume_stream_code)
    
    # fix trait visibility issues
    client_content = client_content.replace("pub(crate) fn deref(", "fn deref(")
    client_content = client_content.replace("pub(crate) fn deref_mut(", "fn deref_mut(")
    client_content = client_content.replace("pub(crate) fn drop(", "fn drop(")
    
    with open(CLIENT_FILE, 'w') as f:
        f.write(client_content)

with open(EXECUTOR_FILE, 'r') as f:
    executor_content = f.read()
executor_content = executor_content.replace("pub(crate) fn execute(", "fn execute(")
with open(EXECUTOR_FILE, 'w') as f:
    f.write(executor_content)

print("Fixed.")
