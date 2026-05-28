
use crate::*;
use api::*;
use runtime::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;

pub(crate) struct CliToolExecutor {
    pub(crate) renderer: TerminalRenderer,
    pub(crate) emit_output: bool,
    pub(crate) allowed_tools: Option<AllowedToolSet>,
    pub(crate) tool_registry: GlobalToolRegistry,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
}

impl CliToolExecutor {
    pub(crate) fn new(
        allowed_tools: Option<AllowedToolSet>,
        emit_output: bool,
        tool_registry: GlobalToolRegistry,
        mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
    ) -> Self {
        Self {
            renderer: TerminalRenderer::new(),
            emit_output,
            allowed_tools,
            tool_registry,
            mcp_state,
        }
    }

    pub(crate) fn execute_search_tool(&self, value: serde_json::Value) -> Result<String, ToolError> {
        let input: ToolSearchRequest = serde_json::from_value(value)
            .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
        let (pending_mcp_servers, mcp_degraded) =
            self.mcp_state.as_ref().map_or((None, None), |state| {
                let state = state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (state.pending_servers(), state.degraded_report())
            });
        serde_json::to_string_pretty(&self.tool_registry.search(
            &input.query,
            input.max_results.unwrap_or(5),
            pending_mcp_servers,
            mcp_degraded,
        ))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn execute_runtime_tool(
        &self,
        tool_name: &str,
        value: serde_json::Value,
    ) -> Result<String, ToolError> {
        let Some(mcp_state) = &self.mcp_state else {
            return Err(ToolError::new(format!(
                "runtime tool `{tool_name}` is unavailable without configured MCP servers"
            )));
        };
        let mut mcp_state = mcp_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match tool_name {
            "MCPTool" => {
                let input: McpToolRequest = serde_json::from_value(value)
                    .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
                let qualified_name = input
                    .qualified_name
                    .or(input.tool)
                    .ok_or_else(|| ToolError::new("missing required field `qualifiedName`"))?;
                mcp_state.call_tool(&qualified_name, input.arguments)
            }
            "ListMcpResourcesTool" => {
                let input: ListMcpResourcesRequest = serde_json::from_value(value)
                    .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
                match input.server {
                    Some(server_name) => mcp_state.list_resources_for_server(&server_name),
                    None => mcp_state.list_resources_for_all_servers(),
                }
            }
            "ReadMcpResourceTool" => {
                let input: ReadMcpResourceRequest = serde_json::from_value(value)
                    .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
                mcp_state.read_resource(&input.server, &input.uri)
            }
            _ => mcp_state.call_tool(tool_name, Some(value)),
        }
    }
}

impl ToolExecutor for CliToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        if self
            .allowed_tools
            .as_ref()
            .is_some_and(|allowed| !allowed.contains(tool_name))
        {
            return Err(ToolError::new(format!(
                "tool `{tool_name}` is not enabled by the current --allowedTools setting"
            )));
        }
        let value = serde_json::from_str(input)
            .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
        let result = if tool_name == "ToolSearch" {
            self.execute_search_tool(value)
        } else if self.tool_registry.has_runtime_tool(tool_name) {
            self.execute_runtime_tool(tool_name, value)
        } else {
            self.tool_registry
                .execute(tool_name, &value)
                .map_err(ToolError::new)
        };
        match result {
            Ok(output) => {
                if self.emit_output {
                    let markdown = format_tool_result(tool_name, &output, false);
                    self.renderer
                        .stream_markdown(&markdown, &mut io::stdout())
                        .map_err(|error| ToolError::new(error.to_string()))?;
                }
                Ok(output)
            }
            Err(error) => {
                if self.emit_output {
                    let markdown = format_tool_result(tool_name, &error.to_string(), true);
                    self.renderer
                        .stream_markdown(&markdown, &mut io::stdout())
                        .map_err(|stream_error| ToolError::new(stream_error.to_string()))?;
                }
                Err(error)
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
        fn tool_rendering_helpers_compact_output() {
            let start = format_tool_call_start("read_file", r#"{"path":"src/main.rs"}"#);
            assert!(start.contains("read_file"));
            assert!(start.contains("src/main.rs"));
    
            let done = format_tool_result(
                "read_file",
                r#"{"file":{"filePath":"src/main.rs","content":"hello","numLines":1,"startLine":1,"totalLines":1}}"#,
                false,
            );
            assert!(done.contains("📄 Read src/main.rs"));
            assert!(done.contains("hello"));
        }

    #[test]
        fn tool_rendering_truncates_large_read_output_for_display_only() {
            let content = (0..200)
                .map(|index| format!("line {index:03}"))
                .collect::<Vec<_>>()
                .join("\n");
            let output = json!({
                "file": {
                    "filePath": "src/main.rs",
                    "content": content,
                    "numLines": 200,
                    "startLine": 1,
                    "totalLines": 200
                }
            })
            .to_string();
    
            let rendered = format_tool_result("read_file", &output, false);
    
            assert!(rendered.contains("line 000"));
            assert!(rendered.contains("line 079"));
            assert!(!rendered.contains("line 199"));
            assert!(rendered.contains("full result preserved in session"));
            assert!(output.contains("line 199"));
        }

    #[test]
        fn tool_rendering_truncates_large_bash_output_for_display_only() {
            let stdout = (0..120)
                .map(|index| format!("stdout {index:03}"))
                .collect::<Vec<_>>()
                .join("\n");
            let output = json!({
                "stdout": stdout,
                "stderr": "",
                "returnCodeInterpretation": "completed successfully"
            })
            .to_string();
    
            let rendered = format_tool_result("bash", &output, false);
    
            assert!(rendered.contains("stdout 000"));
            assert!(rendered.contains("stdout 059"));
            assert!(!rendered.contains("stdout 119"));
            assert!(rendered.contains("full result preserved in session"));
            assert!(output.contains("stdout 119"));
        }

    #[test]
        fn tool_rendering_truncates_generic_long_output_for_display_only() {
            let items = (0..120)
                .map(|index| format!("payload {index:03}"))
                .collect::<Vec<_>>();
            let output = json!({
                "summary": "plugin payload",
                "items": items,
            })
            .to_string();
    
            let rendered = format_tool_result("plugin_echo", &output, false);
    
            assert!(rendered.contains("plugin_echo"));
            assert!(rendered.contains("payload 000"));
            assert!(rendered.contains("payload 040"));
            assert!(!rendered.contains("payload 080"));
            assert!(!rendered.contains("payload 119"));
            assert!(rendered.contains("full result preserved in session"));
            assert!(output.contains("payload 119"));
        }

    #[test]
        fn tool_rendering_truncates_raw_generic_output_for_display_only() {
            let output = (0..120)
                .map(|index| format!("raw {index:03}"))
                .collect::<Vec<_>>()
                .join("\n");
    
            let rendered = format_tool_result("plugin_echo", &output, false);
    
            assert!(rendered.contains("plugin_echo"));
            assert!(rendered.contains("raw 000"));
            assert!(rendered.contains("raw 059"));
            assert!(!rendered.contains("raw 119"));
            assert!(rendered.contains("full result preserved in session"));
            assert!(output.contains("raw 119"));
        }

    #[test]
        fn short_tool_id_truncates_long_identifiers_with_ellipsis() {
            // given
            let long = "toolu_01ABCDEFGHIJKLMN";
            let short = "tool_1";
    
            // when
            let trimmed_long = short_tool_id(long);
            let trimmed_short = short_tool_id(short);
    
            // then
            assert_eq!(trimmed_long, "toolu_01ABCD…");
            assert_eq!(trimmed_short, "tool_1");
        }
}
