# 1. 问题

`claw-cli` 当前把 CLI 入口定义拆成了两套：`src/main.rs:172-321` 的手写 `parse_args()` 是实际生效入口，而 `src/args.rs:5-50`、`src/app.rs:10-15` 仍保留了另一套未接线的原型。再加上 `Cargo.toml:12-24` 并未声明 `clap`，这套原型事实上不在主执行链上，参数语义只能靠人工保持同步，已经出现漂移。

## 1.1. **入口定义分叉**

问题范围主要在 `src/main.rs:172-321`、`src/main.rs:4045-4099`、`src/args.rs:5-50`。

`main.rs` 里直接手写分支，帮助文案也单独维护；`args.rs` 又声明了另一套参数模型，包含当前实际入口没有接入的 `config` 和 `ndjson`。这样一来，新增参数时开发者很容易只改一处，另一处继续“看起来合法”。这不是语法问题，而是单一事实来源缺失。

```rust
match args[index].as_str() {
    "--output-format" => { /* 手写取值 */ }
    "--permission-mode" => { /* 手写取值 */ }
    "--allowedTools" | "--allowed-tools" => { /* 手写别名 */ }
    other => rest.push(other.to_string()),
}
```

```rust
#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,
}

pub enum OutputFormat { Text, Json, Ndjson }
```

## 1.2. **解析与归一化耦合**

问题范围主要在 `src/main.rs:173-321`、`src/main.rs:365-467`。

当前 `parse_args()` 不只负责“把字符串拆成参数”，还同时处理模型别名、权限模式归一化、工具白名单展开、`--resume` 校验、直接 slash 命令分发。也就是说，词法解析、语义归一化、业务动作构造被揉在一个函数里。短期看能工作，但分支继续增长后，阅读和回归都会越来越重。

## 1.3. **测试保护网分裂**

问题范围主要在 `src/main.rs:4154-4396`、`src/args.rs:58-104`、`src/app.rs:360-401`。

真实入口的测试在 `main.rs`，原型入口又有一套 `clap` 解析测试，还带着 `ndjson`、`config` 这类未生效能力。测试数量看起来不少，但并没有共同约束同一套公共行为。结果就是：原型测试通过，不代表 `claw` 真能这么用；真实入口变了，也不会提醒原型已经过时。

# 2. 收益

把 CLI 参数声明收敛到一处后，最直接的收益是：后续新增或调整参数时，帮助文案、解析行为和测试会一起变化，不再靠人工记忆兜底。

## 2.1. **减少修改点**

当前至少有 `parse_args()`、帮助输出、原型类型定义三处入口信息。收敛后可压缩为“声明一处 + 映射一处”，新增参数的核心修改点可从 **3 处** 降到 **1 到 2 处**。

## 2.2. **降低解析复杂度**

`parse_args()` 现在承担了十多个分支和多段后置校验。改成“声明式解析 + 映射到 `CliAction`”后，主函数里的解析包装预计可收敛到 **20 到 40 行**，复杂逻辑转移到具名类型和小函数里，阅读成本更低。

## 2.3. **回归更聚焦**

统一测试后，回归对象会变成“公开 CLI 表面”而不是“两个互不约束的实现”。这能更早发现参数兼容性问题，尤其是别名、默认值和非法输入提示。

# 3. 方案

采用保守改造：保留现有 `CliAction` 和运行分发逻辑，只把“参数声明与解析入口”收敛到一处，不同时重写运行时流程。

## 3.1. **激活** **`args.rs`** **作为唯一入口声明：解决“入口定义分叉”**

方案概述：让 `args.rs` 成为唯一的 CLI 语法定义，`main.rs` 只消费解析结果；`app.rs` 若短期不接入，移出主源码目录或删除，避免继续携带过期语义。

- 在 `Cargo.toml` 补充 `clap` 依赖，并在 `main.rs` 中显式 `mod args;`。
- 先按当前真实能力重写 `args.rs`，只保留已支持的参数和子命令。
- `config`、`ndjson` 这类未落地能力不要继续暴露在默认入口；需要保留时可先放到独立实验模块。
- 帮助文案优先复用 `clap` 元数据，减少 `print_help_to()` 中的手写描述。

修改前：

```rust
fn parse_args(args: &[String]) -> Result<CliAction, String> {
    // 手写扫描所有参数
}
```

修改后：

```rust
#[derive(clap::Parser)]
pub struct CliArgs {
    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output_format: OutputFormat,
    #[arg(long = "allowed-tools", value_delimiter = ',')]
    allowed_tools: Vec<String>,
    #[command(subcommand)]
    command: Option<Command>,
}
```

这一步的重点不是“全面换框架”，而是先把对外参数表面收拢。团队以后查 CLI 行为时，只需要看一处定义。

## 3.2. **保留** **`CliAction`** **但拆分解析阶段：解决“解析与归一化耦合”**

方案概述：继续使用现有 `CliAction` 作为运行边界，但把“字符串解析”和“业务语义归一化”拆成两层。这样既不动核心执行逻辑，又能让入口更清晰。

- `CliArgs::try_parse_from(...)` 负责语法层解析。
- `impl TryFrom<CliArgs> for CliAction` 负责模型别名、工具白名单、`--resume` 等规则归一化。
- `parse_args()` 保留为薄包装，降低调用方改动范围。

修改前：

```rust
match rest[0].as_str() {
    "system-prompt" => parse_system_prompt_args(&rest[1..]),
    "prompt" => Ok(CliAction::Prompt { /* 组装动作 */ }),
    other if other.starts_with('/') => parse_direct_slash_cli_action(&rest),
    _ => Ok(CliAction::Prompt { /* 裸提示词 */ }),
}
```

修改后：

```rust
fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let cli = CliArgs::try_parse_from(
        std::iter::once("claw").chain(args.iter().map(String::as_str))
    ).map_err(|e| e.to_string())?;
    cli.try_into()
}
```

```mermaid
flowchart LR
    A[原始参数] --> B[CliArgs]
    B --> C[规则归一化]
    C --> D[CliAction]
    D --> E[run 分发]
```

这张图展示的是改造后的最小责任链。真正变化的只有前两段，后面的 `run` 分发可以保持稳定，所以这是一种风险较低的渐进式重构。

## 3.3. **合并测试并清理休眠原型：解决“测试保护网分裂”**

方案概述：测试只围绕公开 CLI 行为编写，废弃原型专属断言；`app.rs` 若仍需保留，应改为示例或实验代码，不再占据主 crate 的 `src` 根目录。

- 把解析测试集中到 `args.rs` 或 `tests/cli_args.rs`，统一断言 `CliArgs -> CliAction` 的最终结果。
- 删除针对未公开能力的默认测试，例如 `ndjson`、`config`。
- 为 `--help`、`--version`、`--resume`、别名参数补充负例测试，确保错误提示稳定。

修改前：

```rust
// args.rs
assert_eq!(cli.output_format, OutputFormat::Ndjson);

// main.rs
assert_eq!(parse_args(&args)?, CliAction::Prompt { /* json */ });
```

修改后：

```rust
#[test]
fn parses_prompt_from_public_cli_surface() {
    let cli = CliArgs::try_parse_from([
        "claw", "--output-format", "json", "prompt", "hello"
    ]).unwrap();
    assert!(matches!(cli.try_into(), Ok(CliAction::Prompt { .. })));
}
```

# 4. 回归范围

这次改动虽然聚焦 CLI 入口，但影响的是用户进入 `claw` 的第一步，所以回归要从“命令是否还能按原有方式启动”来验证，而不是只看某个解析函数是否返回成功。

## 4.1. 主链路

- 无参数启动 `claw`，确认仍进入交互模式，默认模型和默认权限模式不变。
- `claw prompt "hello"`、`claw "hello"`、`claw --output-format json prompt "hello"`，确认非交互输出行为一致。
- `claw agents`、`claw skills`、`claw login`、`claw init`，确认已有子命令入口不受影响。
- `claw --resume session.json /status`，确认会话恢复和 slash 命令校验仍正常。

## 4.2. 边界情况

- 非法参数值：如错误的 `--permission-mode`、错误的 `--output-format`，确认提示信息清晰，且与帮助文案一致。
- 参数别名：`--allowedTools` 与 `--allowed-tools`、`--model=opus` 这类兼容写法要继续可用，避免破坏已有脚本。
- 缺失参数值：`--model`、`--resume`、`system-prompt --date` 等缺值场景要稳定报错，不能退化成裸提示词。
- 未公开能力：确认 `config`、`ndjson` 不会再出现在默认帮助或测试里，避免用户误以为已经支持。

