## 项目介绍
  本项目是claude code 的rust 版本，基于 https://github.com/ultraworkers/claw-code.git 分叉而来。提供可直接下载执行的文件。目前支持linux, macOS 系统。

## 快速入手
  1. 在 https://github.com/davids-dot/claw-code/releases 页面下载对应环境的压缩包。
  2. 解压，把解压产生的文件重命名为claw
  3.  运行

### 运行方式
 1. 使用 百炼平台的apiKey, 模型 glm-5.1 
  ```
    export DASHSCOPE_API_KEY=sk-your-key
    export ANTHROPIC_MODEL="glm-5.1"

    ./claw  
  ```

2. 使用deepseek的apiKey, 模型 qwen-plus 
  ```
    export DEEPSEEK_API_KEY=sk-your-key
    export DEEPSEEK_MODEL="deepseek-v4-flash"

    ./claw  
  ```