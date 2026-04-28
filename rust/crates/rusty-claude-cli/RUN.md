# 报错 `missing_credentials` 调用链及业务逻辑分析

当运行 `cargo run --package rusty-claude-cli --bin claw` 时，如果没有配置相应的环境变量，会报 `[error-kind: missing_credentials]` 错误。以下是完整的调用链及业务逻辑分析：

## 1. 调用链 (Call Chain)

1. **`rusty-claude-cli/src/main.rs:202`**: `main()` 函数作为入口，调用 `run()`。
2. **`rusty-claude-cli/src/main.rs:335`**: `run()` 函数解析命令行参数。如果匹配到需要与模型交互的动作（如 `Prompt` 或 `Repl`），则调用 `LiveCli::new(...)` 实例化客户端。

3. **`rusty-claude-cli/src/main.rs:4134`**: `LiveCli::new()` 内部调用 `build_runtime(...)` 构建会话运行时。
4. **`rusty-claude-cli/src/main.rs:7252`**: `build_runtime()` 调用 `build_runtime_with_plugin_state(...)`。
5. **`rusty-claude-cli/src/main.rs:7280`**: `build_runtime_with_plugin_state()` 中实例化 `AnthropicRuntimeClient::new(...)` 以创建 API 提供商客户端。
6. **`rusty-claude-cli/src/main.rs:7433`**: `AnthropicRuntimeClient::new()` 会在匹配到 `ProviderKind::Anthropic`（默认）时，调用 `resolve_cli_auth_source()`。
7. **`rusty-claude-cli/src/main.rs:7504`**: `resolve_cli_auth_source()` 调用内部函数 `resolve_cli_auth_source_for_cwd()`。
8. **`rusty-claude-cli/src/main.rs:7508`**: `resolve_cli_auth_source_for_cwd()` 跨 crate 调用 `api::resolve_startup_auth_source(|| Ok(None))`。
9. **`api/src/providers/anthropic.rs:656`**: `resolve_startup_auth_source()` 尝试从环境变量或 `.env` 文件中读取 `ANTHROPIC_API_KEY` 和 `ANTHROPIC_AUTH_TOKEN`。
10. **`api/src/providers/mod.rs:394`**: 如果上述两个环境变量均不存在，调用 `anthropic_missing_credentials()` 并返回 `Err`。
11. **错误向上冒泡**: `Err` 通过 `?` 操作符层层冒泡，最终被 `main()` 函数捕获。
12. **`rusty-claude-cli/src/main.rs:202`**: `main()` 函数捕获错误后，调用 `classify_error_kind(&message)` 将错误归类为 `"missing_credentials"`，然后将格式化后的错误 `[error-kind: missing_credentials]` 打印到终端并退出。

## 2. 业务逻辑 (Business Logic)

**前置校验（Fail-Fast）**
Claw 命令行工具的设计采用了“快速失败（Fail-Fast）”的策略。在进入 REPL（交互式命令行）或处理 Prompt 之前，它必须确保有权访问大模型 API（默认是 Anthropic）。因此，在构建核心的 `ConversationRuntime` 和 API Client 阶段，它会主动（eagerly）去检测鉴权凭证是否存在。

这样做的好处是避免了用户可能输入了一大段 Prompt 或者开启了会话后，在实际发起 HTTP 请求时才报错，提供了更好的用户体验。

**鉴权方式的优先级**
在 `resolve_startup_auth_source` 的逻辑中：
- 优先检查 `ANTHROPIC_API_KEY`。如果存在，还会进一步检查是否有 `ANTHROPIC_AUTH_TOKEN`，如果有则组成 `ApiKeyAndBearer`，否则仅使用 `ApiKey`。
- 如果没有 `API_KEY`，则检查是否有 `ANTHROPIC_AUTH_TOKEN`，如果有则使用 `BearerToken` 方式。
- 如果二者都读取不到（无论从系统环境变量还是当前目录的 `.env` 文件），则直接抛出凭证缺失错误。

## 3. 其他修改/绕过方式的可能思路

如果你不想直接在系统中配置环境变量，基于上述代码逻辑，有以下几种替代方案：
1. **使用 `.env` 文件**：底层 `read_env_non_empty` 函数会 fallback 到 `super::dotenv_value(key)`，这意味着你可以在项目运行目录（如 `rust/crates/rusty-claude-cli` 或 Workspace 根目录）创建一个 `.env` 文件，在里面写上 `ANTHROPIC_API_KEY=xxx`，程序也能读取到。
2. **切换 Provider / Model**：`AnthropicRuntimeClient::new()` 内部通过 `detect_provider_kind(&resolved_model)` 进行路由。如果在启动时传入 `--model openai/gpt-4` 或者通过 `.claw.json` 配置了其他模型（如 OpenAI、xAI 等），程序会进入其他的 Client 初始化分支，从而绕过针对 Anthropic 环境变量的强校验（当然，它会转而去校验相应提供商的环境变量，如 `OPENAI_API_KEY`）。
3. **Mock Server 测试模式**：在测试用例中，代码会读取 `ANTHROPIC_BASE_URL` 指向本地 Mock 服务器。如果你在做纯离线开发，也许可以考虑走测试桩（Mock）的逻辑，但这通常只在单元测试（如 `tests/compact_output.rs`）中适用。
