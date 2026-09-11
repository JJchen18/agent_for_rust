use std::fmt;

/// 贯穿整个 agent 的错误类型。
#[derive(Debug)]
pub enum AgentError {
    Io(std::io::Error),
    /// 模型点名了一个不存在的工具。
    UnknownTool(String),
    /// `CALL:` 行格式有问题。
    BadToolCall { line: String, reason: String },
    /// 达到最大迭代次数仍未给出最终答案。
    MaxIterationsReached(usize),
    /// 后端（模型）相关错误。
    Backend(String),
    /// 配置 / 命令行参数错误。
    Config(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::Io(e) => write!(f, "io 错误: {e}"),
            AgentError::UnknownTool(t) => write!(f, "未知工具: {t:?}"),
            AgentError::BadToolCall { line, reason } => {
                write!(f, "工具调用行 {line:?} 解析失败: {reason}")
            }
            AgentError::MaxIterationsReached(n) => {
                write!(f, "达到最大迭代次数 {n} 仍未得到最终答案")
            }
            AgentError::Backend(msg) => write!(f, "后端错误: {msg}"),
            AgentError::Config(msg) => write!(f, "配置错误: {msg}"),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<std::io::Error> for AgentError {
    fn from(e: std::io::Error) -> Self {
        AgentError::Io(e)
    }
}

/// 项目内统一使用的 Result 别名。
pub type Result<T> = std::result::Result<T, AgentError>;
