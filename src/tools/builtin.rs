use crate::error::{AgentError, Result};
use super::{Tool, ToolRegistry};
use std::fs;
use std::time::SystemTime;

/// `get_time` —— 无参数，返回当前本地时间。
pub struct GetTime;
impl Tool for GetTime {
    fn name(&self) -> &str {
        "get_time"
    }
    fn description(&self) -> &str {
        "返回当前本地日期与时间（Unix 秒）。无参数。"
    }
    fn call(&self, _args: &[String]) -> Result<String> {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Ok(format!("当前 Unix 时间戳（秒）: {secs}"))
    }
}

/// `echo <...>` —— 原样返回参数（用空格连接）。
pub struct Echo;
impl Tool for Echo {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "回显其参数（空格连接）。参数: 要回显的文本。"
    }
    fn call(&self, args: &[String]) -> Result<String> {
        Ok(args.join(" "))
    }
}

/// `read_file <path>` —— 读取磁盘上的文本文件，截断到前 N 个字符。
pub struct ReadFile {
    max_chars: usize,
}
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "从磁盘读取一个文本文件并返回其内容（截断）。参数: <文件路径>。"
    }
    fn call(&self, args: &[String]) -> Result<String> {
        let Some(path) = args.first() else {
            return Err(AgentError::BadToolCall {
                line: "read_file".into(),
                reason: "需要一个参数: <文件路径>".into(),
            });
        };
        let s = fs::read_to_string(path)?;
        let total = s.chars().count();
        let truncated: String = s.chars().take(self.max_chars).collect();
        Ok(if total > self.max_chars {
            format!("{truncated}…（已截断，共 {total} 字符）")
        } else {
            truncated
        })
    }
}

/// 向注册表添加全部内置工具。
pub fn register_all(reg: &mut ToolRegistry) {
    reg.register(Box::new(GetTime));
    reg.register(Box::new(Echo));
    reg.register(Box::new(ReadFile { max_chars: 2000 }));
}
