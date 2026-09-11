use super::LlmBackend;
use crate::error::Result;

/// 确定性、零依赖的后端，用于在没有模型的情况下端到端跑通整个 agent 循环。
///
/// 它识别两个「意图」并据此驱动工具调用，否则原样复述用户输入：
///   - 以 `echo <文本>` 开头  →  先 `CALL: echo <文本>`，拿到 OBSERVATION 后作答
///   - 提到 time / 时间 / 几点  →  先 `CALL: get_time`，拿到 OBSERVATION 后作答
/// 这样 `cargo run -- --ask "现在几点了?"` 会真实地走一遍 模型→工具→观察→作答。
///
/// 关键：判断「本轮是否已拿到工具结果」时，只看在**最近一条 user 行之后**出现的
/// OBSERVATION，避免多轮对话里把上一轮的旧观察误用到新一轮。
pub struct MockBackend;

impl MockBackend {
    pub fn new() -> Self {
        Self
    }
}

impl LlmBackend for MockBackend {
    fn name(&self) -> &str {
        "mock"
    }

    fn generate(&mut self, transcript: &str) -> Result<String> {
        let last_user = last_user_line(transcript).unwrap_or_default();
        let lower = last_user.to_lowercase();
        let obs = observation_after_last_user(transcript);

        // 意图一：echo <文本>（前缀匹配，更具体，优先判断）。
        if let Some(payload) = lower.strip_prefix("echo ") {
            let payload = payload.trim();
            if !payload.is_empty() {
                return match obs {
                    Some(res) => Ok(format!("ANSWER: {res}")),
                    None => Ok(format!("CALL: echo {payload}")),
                };
            }
        }

        // 意图二：当前时间。
        if lower.contains("time") || lower.contains("时间") || lower.contains("几点") {
            return match obs {
                Some(res) => Ok(format!("ANSWER: {res}")),
                None => Ok("CALL: get_time".to_string()),
            };
        }

        // 默认：确认收到并复述。
        Ok(format!("ANSWER: (mock) 我收到: {last_user:?}"))
    }
}

/// 找到对话记录里最后一行 `user: ...` 的内容。
fn last_user_line(transcript: &str) -> Option<&str> {
    transcript
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix("user: "))
}

/// 只取「最近一条 user 行之后」出现的 `tool: OBSERVATION: <结果>`。
/// 这样多轮对话不会把上一轮的旧观察误用到新一轮。
fn observation_after_last_user(transcript: &str) -> Option<String> {
    let lines: Vec<&str> = transcript.lines().collect();
    let mut last_user_idx = 0usize;
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("user: ") {
            last_user_idx = i;
        }
    }
    lines
        .iter()
        .skip(last_user_idx)
        .find_map(|l| l.strip_prefix("tool: OBSERVATION: ").map(|s| s.to_string()))
}
