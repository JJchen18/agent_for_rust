mod mock;

use crate::config::{AppConfig, BackendKind};
use crate::error::{AgentError, Result};

pub use mock::MockBackend;

/// 语言模型后端。
///
/// 实现必须是「会话无状态」的：每次调用都传入完整对话记录（transcript）。
/// 这与本地 llama.cpp 推理的工作方式一致 —— 模型本身不保留上下文，
/// 上下文由调用方每轮重新喂进去。
pub trait LlmBackend {
    /// 面向用户/日志的后端名。
    fn name(&self) -> &str;
    /// 给定完整对话记录，产出模型的下一步（应遵循系统提示词里的 CALL:/ANSWER: 协议）。
    fn generate(&mut self, transcript: &str) -> Result<String>;
}

/// 按配置构造后端。
pub fn build_backend(cfg: &AppConfig) -> Result<Box<dyn LlmBackend>> {
    match cfg.backend {
        BackendKind::Mock => Ok(Box::new(MockBackend::new())),
        BackendKind::Llama => {
            // ── 接线后改成（见 templates/llama_backend.rs 文件头步骤）：
            // let path = cfg.model_path.clone().ok_or_else(|| {
            //     AgentError::Config("--model <path.gguf> 为 llama 后端必需".into())
            // })?;
            // Ok(Box::new(llama::LlamaBackend::load(path)?))
            Err(AgentError::Backend(
                "llama 后端在本构建中未启用：它依赖 crates.io 上的 llama-cpp-rs，\
                 而本机公司代理（proxyhk.huawei.com）当前拦截 crates.io。\
                 待网络恢复后，按 README.md「启用 llama 后端」与 templates/llama_backend.rs 接线。"
                    .to_string(),
            ))
        }
    }
}
