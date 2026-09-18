# agent_test —— 端侧 Rust Agent 骨架

一个用 Rust 写的、可跑在端侧（本机 / 设备）的最小 agent 骨架。

- **纯 std、零外部依赖**：完全离线可编译、可运行，无供应链、无网络需求。
- **agent 循环**：模型 → 工具调用 → 观察 → 模型 …，直到产出最终答案。
- **后端可插拔**：默认是确定性的 `mock` 后端（无需模型即可演示 / 测试循环）；
  真正的本地推理后端 `llama`（llama-cpp-rs）已留好接缝，待网络可达后一键接线。
- **工具注册表**：内置 `get_time` / `echo` / `read_file`，易于扩展。

## 为什么是「零依赖」
本机的公司代理（`proxyhk.huawei.com`）当前 403 / SSL 拦截 crates.io，任何外部
crate 都拉不下来。所以骨架刻意只用 std：既能立刻编译运行，又恰好是端侧部署的
加分项（二进制小、无运行时、无供应链）。一旦 crates.io 可达，加依赖也很容易。

## 快速开始
```powershell
cargo build
cargo run -- --ask "现在几点了?"     # 一次性：mock 会调用 get_time 工具
cargo run -- --demo                  # 演示 harness / agent-core / llm 三层结构
cargo run                            # 交互式 REPL
```
示例输出（mock 后端真实走了一遍工具循环）：
```
> 现在几点了?
当前 Unix 时间戳（秒）: 1789091041
> echo hi repl
hi repl
> 你好
(mock) 我收到: "你好"
```

## 命令行
| 选项 | 说明 | 默认 |
|---|---|---|
| `--backend <mock\|llama>` | 推理后端 | `mock` |
| `--model <path.gguf>` | GGUF 模型路径（llama 用） | — |
| `--system <文本>` | 自定义系统提示词 | 内置协议提示词 |
| `--max-iters <n>` | 每轮最多工具迭代次数 | `5` |
| `--ask <问题>` | 一次性回答后退出（否则进 REPL） | — |
| `--demo` | 演示 harness / agent-core / llm 三层结构 | — |
| `-h, --help` | 帮助 | |

## agent 循环与工具协议
模型每步只输出两种行之一（见 `src/agent.rs` 的默认系统提示词）：
- `CALL: <工具> [参数...]` —— 调用工具；参数按空白切分。
- `ANSWER: <最终回答>` —— 收尾。

agent 循环（`Agent::run_turn`）：渲染完整 transcript → 问后端 → 若为 `CALL:`
则查 `ToolRegistry` 执行、把结果以 `OBSERVATION: <结果>` 追加进 transcript 再循环；
若为 `ANSWER:`（或无前缀的整段输出）则返回。跨轮 `history` 累积以保留上下文。
这种「文本协议 + 观察回填」的做法对本地小模型比严格 JSON 更稳。

## 三层结构：harness / agent-core / llm

```
harness     组装与驱动（CLI/REPL、配置、I/O）              main.rs / config.rs
   | 组装 + 驱动
agent-core  推理循环（CALL/ANSWER 协议、工具分发、历史）     agent.rs / tools/
   | 只依赖 LlmBackend 抽象
llm         大脑（transcript 进 -> 下一步出，无状态、可插拔） backend/
```

依赖自上而下单向：harness 认识 agent-core；agent-core 只认 `LlmBackend`
抽象，不关心背后是 mock 还是 llama；llm 层对上面两层一无所知。
`cargo run -- --demo` 依次演示三层：直接调后端看「文本进 / 文本出」、
裸跑 Agent 循环（tracer 展示每步迭代）、换一个后端跑同一问题做对比。

## 项目结构
```
src/
  main.rs            CLI 解析 + REPL（harness 外壳）
  demo.rs            --demo：harness / agent-core / llm 三层结构演示
  config.rs          AppConfig / BackendKind
  error.rs           AgentError + Result 别名
  agent.rs           Agent 循环、transcript 渲染、CALL/ANSWER 解析（agent-core）
  backend/mod.rs     LlmBackend trait + build_backend 工厂（llm 层抽象）
  backend/mock.rs    确定性 mock 后端（演示 / 测试循环）
  tools/mod.rs       Tool trait + ToolRegistry（agent-core 的「手」）
  tools/builtin.rs   get_time / echo / read_file
templates/
  llama_backend.rs   llama-cpp-rs 接线模板（默认不编译）
```

## 启用 llama 后端（本地推理）
当前 `--backend llama` 会返回清晰错误（未接线），原因是 **crates.io 被本机代理拦截**，
`llama-cpp-rs` 拉不下来。解除方法：

**A. 让 crates.io 可达**
- 修复 / 放行公司代理，或换网络；或在能访问的机器上 `cargo vendor` 出依赖再拷入。

**B. 接线（网络可达后）**
1. `Cargo.toml` 加：
   ```toml
   [dependencies]
   llama-cpp-rs = { version = "0.1", default-features = false, features = ["library-loader"] }
   ```
   `library-loader` = 运行时加载预编译 `llama.dll`，**不需要** CMake / C++ 工具链。
   预编译库从 ggml-org/llama.cpp 的 releases 取，通常用环境变量
   `LLAMA_LIBRARY_PATH`（以 crate 文档为准）指向它。
   若想从源码编译：去掉 `library-loader`，但需先装 CMake + C++ 编译器。
2. 把 `templates/llama_backend.rs` 移到 `src/backend/llama.rs`，
   并在 `src/backend/mod.rs` 加 `mod llama;`。
3. 按模板内注释把 `load()` / `generate()` 的 llama 调用填实
   （API 随版本变，按编译器报错微调）。
4. 把 `build_backend()` 的 `Llama` 分支改为实例化 `llama::LlamaBackend::load(path)`
   （该文件里已留好注释好的实现）。
5. 放一个 GGUF 模型文件，运行：
   ```powershell
   cargo run -- --backend llama --model models/你的模型.gguf
   ```

> 注意：本环境无法在线验证 `llama-cpp-rs` 的确切 API，模板里的 llama 调用是按常见
> 版本写的意图代码，接线后请以你实际拉到的 crate 文档 / 编译器报错为准微调。

## 端侧部署备注
- 纯 Rust 静态二进制、无运行时；`cargo build --release` 即得小体积可执行文件。
- 可交叉编译到其它端侧目标（如 `aarch64` 手机 / 嵌入式），需要对应 target 与 C 库。
- 工具是进程内的 Rust 闭包，天然适合后续沙箱化（加权限 / 白名单）。

## 已知限制 / 下一步
- llama 后端未接线（受网络限制），当前用 mock 演示循环。
- 工具参数为空白切分的 token；如需带空格 / 结构化参数，可升级为 JSON（需 serde）。
- 无流式输出、无并发、无持久化会话；这些是「最小骨架」刻意留白的部分。
- 可扩展：上下文长度管理、工具结果截断策略、REPL 的 `/tools` 查看已注册工具等。
