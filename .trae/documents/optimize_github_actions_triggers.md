# GitHub Actions 触发机制优化计划

为了减少 GitHub Actions 的资源浪费并支持手动执行，我们将合并冗余的工作流，并调整触发策略。

## 当前状态分析
- **[rust.yml](file:///Users/zhaoyanchao/Code/python/claw-code-main/.github/workflows/rust.yml)**: 仅执行基础的编译和测试，与 `rust-ci.yml` 功能重叠，属于冗余配置。
- **[rust-ci.yml](file:///Users/zhaoyanchao/Code/python/claw-code-main/.github/workflows/rust-ci.yml)**: 包含格式检查、代码检查（Clippy）和工作区测试，功能全面，已支持 `workflow_dispatch`。
- **[release.yml](file:///Users/zhaoyanchao/Code/python/claw-code-main/.github/workflows/release.yml)**: 用于发布二进制产物，已支持 `workflow_dispatch`。

## 提议的变更

### 1. 删除冗余工作流
- **文件**: `rust.yml`
- **操作**: 直接删除该文件。其功能（编译和测试）已由 `rust-ci.yml` 的 `test-workspace` 步骤覆盖。

### 2. 优化 CI 触发策略
- **文件**: **[rust-ci.yml](file:///Users/zhaoyanchao/Code/python/claw-code-main/.github/workflows/rust-ci.yml)**
- **操作**:
    - 删除 `on.push` 配置。这将停止在每次代码推送（Push）到 main 或其他匹配分支时自动运行。
    - 保留 `on.pull_request`。确保在提交合并请求时仍能自动进行质量检查，这是保障代码质量的关键。
    - 保留 `on.workflow_dispatch`。允许您在 GitHub Actions 页面随时手动触发完整 CI 流程。

### 3. 检查发布工作流
- **文件**: **[release.yml](file:///Users/zhaoyanchao/Code/python/claw-code-main/.github/workflows/release.yml)**
- **操作**:
    - 该文件已配置 `workflow_dispatch`，允许手动触发发布。
    - 保留 `on.push.tags`。通常发布是基于版本标签触发的，这符合常规发布流程且频率较低，不属于资源浪费。

## 验证步骤
1. 检查 `.github/workflows/` 目录下 `rust.yml` 是否已删除。
2. 查看 `rust-ci.yml` 的代码，确认 `push` 触发器已被移除。
3. 在 GitHub 仓库的 Actions 页面，确认 `Rust CI` 和 `Release binaries` 工作流都出现了 "Run workflow" 的手动执行按钮。
