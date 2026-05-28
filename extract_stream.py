import re
import os

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

consume_stream_pattern = r'    /// Consume a single streaming response, optionally applying a stall\n    /// timeout on the first event for post-tool continuations\.\n    #\[allow\(clippy::too_many_lines\)\]\n    async fn consume_stream\(\n        &self,\n        message_request: &MessageRequest,\n        apply_stall_timeout: bool,\n    \) -> Result<Vec<AssistantEvent>, RuntimeError> \{.*?\n    \}'
consume_stream_code, content = extract_and_remove(consume_stream_pattern, content)

response_to_events_pattern = r'fn response_to_events\(\n    response: MessageResponse,\n    out: &mut \(impl Write \+ \?Sized\),\n\) -> Result<Vec<AssistantEvent>, RuntimeError> \{.*?\n\}'
response_to_events_code, content = extract_and_remove(response_to_events_pattern, content)

push_prompt_cache_record_pattern = r'fn push_prompt_cache_record\(client: &ApiProviderClient, events: &mut Vec<AssistantEvent>\) \{.*?\n\}'
push_prompt_cache_record_code, content = extract_and_remove(push_prompt_cache_record_pattern, content)

prompt_cache_record_to_runtime_event_pattern = r'fn prompt_cache_record_to_runtime_event\(\n    record: api::PromptCacheRecord,\n\) -> Option<PromptCacheEvent> \{.*?\n\}'
prompt_cache_record_to_runtime_event_code, content = extract_and_remove(prompt_cache_record_to_runtime_event_pattern, content)

request_ends_with_tool_result_pattern = r'/// Returns `true` when the conversation ends with a tool-result message,\n/// meaning the model is expected to continue after tool execution\.\nfn request_ends_with_tool_result\(request: &ApiRequest\) -> bool \{.*?\n\}'
request_ends_with_tool_result_code, content = extract_and_remove(request_ends_with_tool_result_pattern, content)

with open(MAIN_FILE, 'w') as f:
    f.write(content)

with open('stream_extracted.rs', 'w') as f:
    if consume_stream_code: f.write(consume_stream_code + "\n\n")
    if response_to_events_code: f.write(response_to_events_code + "\n\n")
    if push_prompt_cache_record_code: f.write(push_prompt_cache_record_code + "\n\n")
    if prompt_cache_record_to_runtime_event_code: f.write(prompt_cache_record_to_runtime_event_code + "\n\n")
    if request_ends_with_tool_result_code: f.write(request_ends_with_tool_result_code + "\n\n")

print("Extraction complete")
