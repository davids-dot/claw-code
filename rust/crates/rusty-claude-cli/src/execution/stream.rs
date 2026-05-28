
use crate::*;
use api::*;
use runtime::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;
pub(crate) fn response_to_events(
    response: MessageResponse,
    out: &mut (impl Write + ?Sized),
) -> Result<Vec<AssistantEvent>, RuntimeError> {
    let mut events = Vec::new();
    let mut pending_tool = None;

    for block in response.content {
        let mut block_has_thinking_summary = false;
        push_output_block(
            block,
            out,
            &mut events,
            &mut pending_tool,
            false,
            &mut block_has_thinking_summary,
        )?;
        if let Some((id, name, input)) = pending_tool.take() {
            events.push(AssistantEvent::ToolUse { id, name, input });
        }
    }

    events.push(AssistantEvent::Usage(response.usage.token_usage()));
    events.push(AssistantEvent::MessageStop);
    Ok(events)
}

pub(crate) fn push_prompt_cache_record(client: &ApiProviderClient, events: &mut Vec<AssistantEvent>) {
    // `ApiProviderClient::take_last_prompt_cache_record` is a pass-through
    // to the Anthropic variant and returns `None` for OpenAI-compat /
    // xAI variants, which do not have a prompt cache. So this helper
    // remains a no-op on non-Anthropic providers without any extra
    // branching here.
    if let Some(record) = client.take_last_prompt_cache_record() {
        if let Some(event) = prompt_cache_record_to_runtime_event(record) {
            events.push(AssistantEvent::PromptCache(event));
        }
    }
}

pub(crate) fn prompt_cache_record_to_runtime_event(
    record: api::PromptCacheRecord,
) -> Option<PromptCacheEvent> {
    let cache_break = record.cache_break?;
    Some(PromptCacheEvent {
        unexpected: cache_break.unexpected,
        reason: cache_break.reason,
        previous_cache_read_input_tokens: cache_break.previous_cache_read_input_tokens,
        current_cache_read_input_tokens: cache_break.current_cache_read_input_tokens,
        token_drop: cache_break.token_drop,
    })
}

/// Returns `true` when the conversation ends with a tool-result message,
/// meaning the model is expected to continue after tool execution.
pub(crate) fn request_ends_with_tool_result(request: &ApiRequest) -> bool {
    request
        .messages
        .last()
        .is_some_and(|message| message.role == MessageRole::Tool)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
        fn response_to_events_preserves_empty_object_json_input_outside_streaming() {
            let mut out = Vec::new();
            let events = response_to_events(
                MessageResponse {
                    id: "msg-1".to_string(),
                    kind: "message".to_string(),
                    model: "claude-opus-4-6".to_string(),
                    role: "assistant".to_string(),
                    content: vec![OutputContentBlock::ToolUse {
                        id: "tool-1".to_string(),
                        name: "read_file".to_string(),
                        input: json!({}),
                    }],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: Usage {
                        input_tokens: 1,
                        output_tokens: 1,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                    },
                    request_id: None,
                },
                &mut out,
            )
            .expect("response conversion should succeed");
    
            assert!(matches!(
                &events[0],
                AssistantEvent::ToolUse { name, input, .. }
                    if name == "read_file" && input == "{}"
            ));
        }

    #[test]
        fn response_to_events_preserves_non_empty_json_input_outside_streaming() {
            let mut out = Vec::new();
            let events = response_to_events(
                MessageResponse {
                    id: "msg-2".to_string(),
                    kind: "message".to_string(),
                    model: "claude-opus-4-6".to_string(),
                    role: "assistant".to_string(),
                    content: vec![OutputContentBlock::ToolUse {
                        id: "tool-2".to_string(),
                        name: "read_file".to_string(),
                        input: json!({ "path": "rust/Cargo.toml" }),
                    }],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: Usage {
                        input_tokens: 1,
                        output_tokens: 1,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                    },
                    request_id: None,
                },
                &mut out,
            )
            .expect("response conversion should succeed");
    
            assert!(matches!(
                &events[0],
                AssistantEvent::ToolUse { name, input, .. }
                    if name == "read_file" && input == "{\"path\":\"rust/Cargo.toml\"}"
            ));
        }

    #[test]
        fn response_to_events_renders_collapsed_thinking_summary() {
            let mut out = Vec::new();
            let events = response_to_events(
                MessageResponse {
                    id: "msg-3".to_string(),
                    kind: "message".to_string(),
                    model: "claude-opus-4-6".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        OutputContentBlock::Thinking {
                            thinking: "step 1".to_string(),
                            signature: Some("sig_123".to_string()),
                        },
                        OutputContentBlock::Text {
                            text: "Final answer".to_string(),
                        },
                    ],
                    stop_reason: Some("end_turn".to_string()),
                    stop_sequence: None,
                    usage: Usage {
                        input_tokens: 1,
                        output_tokens: 1,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                    },
                    request_id: None,
                },
                &mut out,
            )
            .expect("response conversion should succeed");
    
            assert!(matches!(
                &events[0],
                AssistantEvent::TextDelta(text) if text == "Final answer"
            ));
            let rendered = String::from_utf8(out).expect("utf8");
            assert!(rendered.contains("▶ Thinking (6 chars hidden)"));
            assert!(!rendered.contains("step 1"));
        }
}
