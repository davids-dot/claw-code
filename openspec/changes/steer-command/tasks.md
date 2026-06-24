## 1. SteerQueue 核心数据结构

- [x] 1.1 在 `rusty-claude-cli/src/` 新建 `steer.rs`，定义 `SteerQueue` 类型别名（`Arc<Mutex<VecDeque<String>>>`）及 `new()`、`push()`、`drain_all()` 方法
- [x] 1.2 为 `SteerQueue` 编写单元测试：push 单条、push 多条顺序、drain 清空、并发 push 安全

## 2. SlashCommand 枚举扩展

- [x] 2.1 在 `commands/src/lib.rs` 的 `SlashCommand` 枚举中添加 `Steer { text: String }` 变体
- [x] 2.2 在 `commands/src/lib.rs` 的 `SLASH_COMMAND_TABLE` 中添加 `/steer` 条目，描述为 "Inject guidance during AI output"
- [x] 2.3 在 `commands/src/lib.rs` 的 `SlashCommand::parse()` 中添加 `/steer <text>` 解析逻辑

## 3. LiveCli 集成

- [x] 3.1 在 `LiveCli` 结构体中添加 `steer_queue: SteerQueue` 字段，在 `new()` 中初始化
- [x] 3.2 在 `repl_commands.rs` 的 `handle_repl_command` 中处理 `SlashCommand::Steer`：将 text push 到 steer_queue，打印确认 `🧭 Steer queued: <text>`
- [x] 3.3 修改 `live_cli.rs` 的 `run_turn()`：注入 steer_queue 中的文本到 user input

## 4. 非阻塞 stdin 轮询（AI 输出期间输入）

- [x] 4.1 在 `steer.rs` 中实现 `poll_steer_input()` 函数（placeholder，核心路径通过 REPL prompt）
- [ ] 4.2 在 `run_turn()` 的 spinner 渲染循环中，每个 tick 间隙调用 `poll_steer_input()`
- [ ] 4.3 处理 crossterm raw mode 进入/退出，确保不影响 spinner 渲染

> **Note**: Tasks 4.2 和 4.3 需要 `runtime.run_turn()` 支持异步/hook 机制，当前架构为同步阻塞调用。
> 核心功能（REPL 空闲时 `/steer` 输入 + 下次 turn 注入）已完整实现。
> AI 输出期间的实时轮询作为未来增强，需 runtime crate 配合。

## 5. 提示与帮助

- [x] 5.1 在 `input.rs`/`args.rs` 的 `repl_completion_candidates` 中添加 `/steer` 完成项
- [x] 5.2 在 `formatting/reports.rs` 的帮助输出中添加 `/steer <text>` 条目（通过 SlashCommandSpec 自动生成）
- [x] 5.3 在 spinner 渲染中添加 dim 色提示 `Type /steer <text> to guide...`

## 6. 测试与验证

- [x] 6.1 编写 `SteerQueue` 并发安全测试（4 项，全部通过）
- [x] 6.2 编写 `SlashCommand::Steer` 解析测试（2 项，全部通过）
- [ ] 6.3 编写 `run_turn` 中 steer 注入集成测试（mock API client，验证 steer 文本出现在 session messages 中）
- [ ] 6.4 手动验证：启动 REPL → 发起对话 → AI 输出期间输入 `/steer` → 确认 AI 下一次工具调用前读入指导
- [x] 6.5 运行 `cargo clippy` 和 `cargo test` 确认无回归（commands: 42 pass, rusty-claude-cli steer: 4 pass, pre-existing failures unaffected）
