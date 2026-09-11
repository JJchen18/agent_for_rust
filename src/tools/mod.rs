mod builtin;

use crate::error::{AgentError, Result};

/// 一个 agent 可以调用的工具。
///
/// 约定：参数是「按空白切分后的 token 列表」（`CALL: <tool> <arg1> <arg2> ...`），
/// 见 `agent::parse_call`。对本地小模型来说，这种简单文本协议比严格 JSON 更稳。
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    /// 用 `args` 执行工具，返回给模型观察的文本。
    fn call(&self, args: &[String]) -> Result<String>;
}

/// 工具注册表。
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// 注册所有内置工具。
    pub fn register_builtins(&mut self) {
        builtin::register_all(self);
    }

    fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    pub fn call(&self, name: &str, args: &[String]) -> Result<String> {
        match self.get(name) {
            Some(t) => t.call(args),
            None => Err(AgentError::UnknownTool(name.to_string())),
        }
    }

    /// 面向模型的工具清单，拼进系统提示词。
    pub fn describe(&self) -> String {
        let mut out = String::new();
        for t in &self.tools {
            out.push_str(&format!("- {} : {}\n", t.name(), t.description()));
        }
        out
    }
}
