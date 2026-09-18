use crate::backend::LlmBackend;
use crate::config::AppConfig;
use crate::error::{AgentError, Result};
use crate::tools::ToolRegistry;

/// 对话中的一条消息。
#[derive(Clone, Debug)]
pub struct Turn {
    pub role: Role,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    /// 工具执行结果，文本形如 `OBSERVATION: <结果>`。
    Tool,
}

/// 端侧 agent（三层结构中的 agent-core 层）：
/// 持有后端与工具，跑「模型 → 工具 → 观察 → 模型 …」循环。
/// 自身不做任何 I/O；需要观察循环内部时，由 harness 注入 tracer 回调。
pub struct Agent {
    backend: Box<dyn LlmBackend>,
    tools: ToolRegistry,
    system: String,
    max_iters: usize,
    /// 可选追踪钩子：循环每一步都汇报给它。agent-core 保持零 I/O，
    /// 怎么展示（打印 / 日志 / UI）由 harness 决定 —— 层间解耦的演示点。
    tracer: Option<Box<dyn FnMut(&str)>>,
}

impl Agent {
    /// 当前后端名（如 "mock" / "llama"），用于日志与提示。
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }

    pub fn new(backend: Box<dyn LlmBackend>, tools: ToolRegistry, cfg: &AppConfig) -> Self {
        let base = cfg
            .system
            .clone()
            .unwrap_or_else(|| default_system_prompt().to_string());
        let mut system = base;
        system.push_str("\n\n可用工具:\n");
        system.push_str(&tools.describe());
        Self {
            backend,
            tools,
            system,
            max_iters: cfg.max_iters,
            tracer: None,
        }
    }

    /// 安装追踪回调（harness 用来观察循环内部，如 `--demo` 的展示）。
    pub fn set_tracer(&mut self, f: Box<dyn FnMut(&str)>) {
        self.tracer = Some(f);
    }

    fn trace(&mut self, msg: &str) {
        if let Some(t) = self.tracer.as_mut() {
            t(msg);
        }
    }

    /// 处理一轮用户输入，返回最终答案。`history` 跨轮累积以保留上下文。
    pub fn run_turn(&mut self, history: &mut Vec<Turn>, user: &str) -> Result<String> {
        history.push(Turn {
            role: Role::User,
            text: user.to_string(),
        });

        for i in 0..self.max_iters {
            let ev = format!(
                "iter {}/{}: 把完整 transcript（system + {} 轮历史）交给后端「{}」",
                i + 1,
                self.max_iters,
                history.len(),
                self.backend.name()
            );
            self.trace(&ev);

            let transcript = render_transcript(&self.system, history);
            let out = self.backend.generate(&transcript)?.trim().to_string();
            history.push(Turn {
                role: Role::Assistant,
                text: out.clone(),
            });

            if let Some(rest) = out.strip_prefix("CALL: ") {
                let (name, args) = parse_call(rest)?;
                let ev = format!("后端输出 CALL -> 执行工具 {name}，参数 {args:?}");
                self.trace(&ev);
                let result = self.tools.call(&name, &args)?;
                let ev = format!("工具结果回填 transcript: OBSERVATION: {result}");
                self.trace(&ev);
                history.push(Turn {
                    role: Role::Tool,
                    text: format!("OBSERVATION: {result}"),
                });
                continue;
            }
            if let Some(ans) = out.strip_prefix("ANSWER: ") {
                self.trace("后端输出 ANSWER -> 循环结束，返回最终答案");
                return Ok(ans.to_string());
            }
            // 无前缀 —— 直接把整段输出当作最终答案。
            self.trace("后端输出无协议前缀 -> 视为最终答案，循环结束");
            return Ok(out);
        }
        Err(AgentError::MaxIterationsReached(self.max_iters))
    }
}

/// 解析 `CALL:` 之后的内容 → (工具名, 参数列表)。参数按空白切分。
fn parse_call(rest: &str) -> Result<(String, Vec<String>)> {
    let rest = rest.trim();
    let mut parts = rest.splitn(2, ' ');
    let name = parts.next().unwrap_or("").trim().to_string();
    let args_str = parts.next().unwrap_or("").trim();
    if name.is_empty() {
        return Err(AgentError::BadToolCall {
            line: rest.to_string(),
            reason: "缺少工具名".into(),
        });
    }
    let args = if args_str.is_empty() {
        Vec::new()
    } else {
        args_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    };
    Ok((name, args))
}

/// 把系统提示词与历史渲染成喂给模型的纯文本 transcript。
fn render_transcript(system: &str, history: &[Turn]) -> String {
    let mut s = String::new();
    s.push_str("system: ");
    s.push_str(system);
    s.push('\n');
    for t in history {
        let prefix = match t.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        s.push_str(prefix);
        s.push_str(": ");
        s.push_str(&t.text);
        s.push('\n');
    }
    s
}

/// 内置协议提示词（追加工具清单前）。
fn default_system_prompt() -> &'static str {
    "你是一个运行在端侧的助手，可以调用工具完成任务。\n\
     调用工具时，只输出一行: CALL: <工具名> [参数...]\n\
     工具结果会以: OBSERVATION: <结果>  的形式返回给你。\n\
     当你准备好回答用户时，只输出一行: ANSWER: <你的最终回答>\n\
     除上述两种行之外不要输出别的内容。"
}
