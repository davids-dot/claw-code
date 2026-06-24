## Why

在 AI 输出或思考时，用户无法实时施加影响。用户只能等 AI 完成当前 turn 后才能给出下一步指令。`/steer` 命令让用户在 AI 流式输出期间追加指导提示词，AI 在下一次 API 调用时自动读取并融入，无需中断当前对话流。

## What Changes

- 新增 `/steer <text>` REPL slash command：用户在 AI 输出/思考期间输入，将指导文本追加到一个 `SteerQueue`
- `LiveCli` 新增 `steer_queue: Arc<Mutex<VecDeque<String>>>` 字段，作为线程安全的待注入指导队列
- `run_turn` 的对话循环中，每次迭代开始时检查 `steer_queue`，将积压的指导文本作为 `user` message 注入 session
- `SlashCommand` 枚举新增 `Steer { text: String }` 变体
- REPL 主循环改为异步输入：AI 输出期间，后台线程监听 stdin，识别 `/steer` 命令并写入队列
- 新增 `/steer` 完成候选项和帮助文档

## Capabilities

### New Capabilities
- `steer-command`: 在 AI 流式输出期间注入指导提示词，不中断当前对话流

### Modified Capabilities
<!-- 无既有 spec 需要修改 -->

## Impact

- **代码**: `rusty-claude-cli/src/live_cli.rs`（核心逻辑）、`repl_commands.rs`（命令处理）、`input.rs`（异步输入）、`commands/src/lib.rs`（SlashCommand 枚举）
- **依赖**: `tokio`（已有）用于异步 stdin 监听；`std::sync::Arc<Mutex<>>` 用于线程安全队列
- **用户体验**: AI 输出期间可输入 `/steer focus on error handling` 实时引导方向
- **风险**: 异步 stdin 需要正确处理终端原始模式，避免与 rustyline 冲突；队列注入时机需保证消息顺序正确
