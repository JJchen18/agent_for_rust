/// 推理后端种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    /// 确定性的脚本化后端，无需模型即可跑通整个 agent 循环（默认）。
    Mock,
    /// 真正的本地推理后端（llama-cpp-rs），启用方式见 README 与 templates/。
    Llama,
}

impl std::str::FromStr for BackendKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(Self::Mock),
            "llama" => Ok(Self::Llama),
            other => Err(format!("未知后端 {other:?}（可选 mock | llama）")),
        }
    }
}

/// 运行时配置（来自命令行 / 环境变量）。
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub backend: BackendKind,
    /// GGUF 模型文件路径（仅 llama 后端使用）。
    pub model_path: Option<String>,
    /// 自定义系统提示词；缺省时用内置协议提示词。
    pub system: Option<String>,
    /// 每轮对话最多允许多少次工具调用迭代。
    pub max_iters: usize,
    /// 一次性模式：给出问题、得到答案后直接退出（否则进入 REPL）。
    pub ask: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend: BackendKind::Mock,
            model_path: None,
            system: None,
            max_iters: 5,
            ask: None,
        }
    }
}
