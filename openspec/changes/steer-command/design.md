## Context

当前 REPL 架构中，`run_repl()` 主循环是同步的：用户输入 → `run_turn()` 阻塞直到 AI 完成 → 等待下一次输入。AI 输出期间用户无法干预。

核心数据流：
```
┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌──────────┐
│ readline │───▶│ run_turn │───▶│ API stream   │───▶│ tool use │──┐
│ (input)  │    │ (sync)   │    │ (blocking)   │    │ (loop)   │  │
└──────────┘    └──────────┘    └──────────────┘    └──────────┘  │
       ▲                                                         │
       │              ┌──────────────┐                          │
       └──────────────│ next iter    │◀─────────────────────────┘
                      └──────────────┘
```

关键约束：
- `run_turn()` 内部的对话循环每轮迭代调用 `api_client.stream()`，这是同步阻塞的
- `ConversationRuntime::run_turn()` 位于 `runtime` crate，不应依赖 CLI 层
- 终端在 AI 输出时被 spinner/render 占用，stdin 不可读

## Goals / Non-Goals

**Goals:**
- 用户在 AI 流式输出期间可输入 `/steer <text>` 追加指导
- 指导文本在下一次 API 调用前自动注入为 user message
- 不中断当前对话流，不影响 AI 当前正在生成的响应
- 实现优雅简洁，最小化侵入性改动

**Non-Goals:**
- 不支持 `/steer` 中断/取消当前 AI 输出（那是 `/cancel` 的职责）
- 不修改 `runtime` crate 的 `ConversationRuntime`（保持 crate 隔离）
- 不实现复杂的异步终端渲染（保持现有 crossterm 模式）

## Decisions

### Decision 1: 使用 `SteerQueue` + `pre-turn injection` 而非运行时拦截

**选择**: 在 `LiveCli` 中维护一个 `steer_queue: SteerQueue`（`Arc<Mutex<VecDeque<String>>>` 的类型别名）。`run_turn()` 的对话循环每次迭代前，从队列取出积压的 steer 文本，追加为 user message 到 session。

**替代方案 A — 运行时注入到 stream**: 在 API stream 期间实时修改 request。不可行，因为 stream 是阻塞调用。

**替代方案 B — 外部进程写文件**: 通过文件系统通信。过于 hacky，竞态条件多。

**理由**: `SteerQueue` 方案最简洁——CLI 层自己管理队列，通过 `run_turn` 循环的天然间隙注入，不需要修改 runtime crate，不需要 async runtime。

### Decision 2: 使用非阻塞 stdin 轮询而非全异步

**选择**: 在 `run_turn()` 的 spinner 循环中，利用 crossterm 的非阻塞 stdin 轮询检测 `/steer` 输入。当检测到 `/steer` 行时，提取文本写入 `SteerQueue`。

**替代方案 A — tokio async stdin**: 引入完整的异步运行时，与当前同步架构冲突大。

**替代方案 B — 专用输入线程**: 启动后台线程读 stdin。与 rustyline 冲突，终端所有权管理复杂。

**理由**: crossterm 非阻塞轮询已经可用（项目已依赖），在 spinner tick 的间隙检测输入，与现有渲染循环天然融合，零额外依赖。

### Decision 3: 消息注入方式 — 在 session messages 中追加

**选择**: 在 `run_turn()` 的循环迭代之间，检查 `steer_queue`。如果有内容，调用 `session.push_user_text(format!("[steer] {text}"))`，AI 在下一次 API 调用时自然读到。

**理由**: 无需修改 `ApiRequest` 或 `ConversationRuntime`，完全在 CLI 层完成。`[steer]` 前缀让 AI 明确这是用户中途追加的指导。

## Risks / Trade-offs

- **[终端原始模式冲突]** → crossterm 非 block 读取需要 enter raw mode，可能影响 spinner 渲染。缓解：仅在 spinner tick 间隙短暂检测，立即退出 raw mode
- **[消息顺序]** → steer message 可能在 AI 正在生成 tool_use 时注入，导致上下文不连贯。缓解：steer 仅在迭代间隙注入，AI 一定能读到完整的前文
- **[用户不知道可以输入]** → AI 输出期间没有输入提示符。缓解：在 spinner 旁显示 `Type /steer <text> to guide...` 提示
