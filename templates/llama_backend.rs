//! 真正的本地推理后端（llama-cpp-rs）——接线参考模板。
//!
//! ⚠️ 本文件位于 `templates/` 下，默认**不参与编译**（未在任何 `mod` 中声明）。
//!    它是一份「拿来即用、但需按你实际拉到的 crate 版本微调」的参考实现，
//!    因为本环境暂时无法访问 crates.io 验证 `llama-cpp-rs` 的确切 API。
//!
//! ── 启用步骤（详见 README.md「启用 llama 后端」）────────────────────────────
//!  1. 让 crates.io 可达（本机公司代理 proxyhk.huawei.com 当前拦截它）。
//!     备选：在能访问 crates.io 的机器上 `cargo vendor` 出依赖再拷过来，
//!           或修复/放行公司代理。
//!  2. `Cargo.toml` 加依赖。`library-loader` 特性 = 运行时 dlopen 预编译库，
//!     **不从源码编译 llama.cpp**，因此不需要 CMake / C++ 工具链：
//!         [dependencies]
//!         llama-cpp-rs = { version = "0.1", default-features = false,
//!                           features = ["library-loader"] }
//!     （版本号以当时 `cargo add llama-cpp-rs` 拉到的最新为准）
//!     并提供预编译的 `llama.dll`（从 ggml-org/llama.cpp 的 releases 取），
//!     运行时用环境变量 `LLAMA_LIBRARY_PATH` 指到它；
//!     或者安装 CMake + C++ 工具链、去掉 `library-loader` 改为从源码编译。
//!  3. 把本文件移到 `src/backend/llama.rs`，并在 `src/backend/mod.rs` 加 `mod llama;`。
//!  4. 在 `build_backend()` 的 `Llama` 分支里实例化它（见该文件内注释）。
//!  5. 放一个 GGUF 模型文件，运行：
//!         cargo run -- --backend llama --model models/你的模型.gguf
//!
//! 注意：`llama-cpp-rs` 的 API 随版本变化。下面 `// llama::` 起的是意图调用，
//!       请以你实际拉到的 crate 文档为准；签名如有出入按编译器报错微调即可。
//!
//! 当前状态：`load()` / `generate()` 返回清晰的“未接线”错误（不会 panic），
//!       等 crates.io 可达后按上面的注释填实现即可。

use crate::backend::LlmBackend;
use crate::error::{AgentError, Result};

pub struct LlamaBackend {
    // 持有 llama-cpp-rs 的模型 / 上下文句柄，例如：
    //   model: llama::Model,
    //   ctx:   llama::Context,
    // 这里先用一个不透明的占位，接线时替换为真实字段。
    _handle: (),
}

impl LlamaBackend {
    pub fn load(model_path: String) -> Result<Self> {
        // ── 待接线：按 llama-cpp-rs 实际 API 填写 ─────────────────────────
        // let model = llama::Model::from_path(
        //     &model_path,
        //     llama::LlamaModelParams::default(),
        // )
        // .map_err(|e| AgentError::Backend(format!("加载模型失败: {e}")))?;
        // let ctx = model
        //     .create_context(llama::LlamaContextParams::default())
        //     .map_err(|e| AgentError::Backend(format!("创建上下文失败: {e}")))?;
        // Ok(Self { model, ctx })
        let _ = model_path;
        Err(AgentError::Backend(
            "llama 后端尚未接线：见 templates/llama_backend.rs 文件头的启用步骤"
                .to_string(),
        ))
    }
}

impl LlmBackend for LlamaBackend {
    fn name(&self) -> &str {
        "llama"
    }

    /// 把整段 transcript（已含系统提示词与 CALL:/ANSWER: 协议）作为 prompt 喂给模型，
    /// 取回它生成的下一步原始文本（含 CALL:/ANSWER: 前缀），原样返回给 agent 循环解析。
    fn generate(&mut self, transcript: &str) -> Result<String> {
        // ── 待接线：按 llama-cpp-rs 实际 API 填写 ─────────────────────────
        // let out = self
        //     .ctx
        //     .generate(transcript, llama::LlamaGenerateParams::default())
        //     .map_err(|e| AgentError::Backend(e.to_string()))?;
        // Ok(out.text().to_string())
        let _ = transcript;
        Err(AgentError::Backend(
            "llama 后端尚未接线：见 templates/llama_backend.rs 文件头的启用步骤"
                .to_string(),
        ))
    }
}
