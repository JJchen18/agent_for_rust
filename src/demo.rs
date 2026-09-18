//! 三层结构演示：harness（外壳）-> agent-core（核心循环）-> llm（模型后端）。
//!
//! ```text
//! +----------------------------------------------+
//! | harness    外壳 / 驱动层                      |  main.rs / config.rs
//! | 解析配置、组装部件、驱动循环、处理 I/O          |
//! +----------------+-----------------------------+
//!                  | 组装 + 驱动
//! +----------------v-----------------------------+
//! | agent-core 核心循环层                         |  agent.rs / tools/
//! | 推理循环、CALL/ANSWER 协议、历史管理、工具分发   |
//! +----------------+-----------------------------+
//!                  | 只依赖 LlmBackend 抽象
//! +----------------v-----------------------------+
//! | llm 模型层                                    |  backend/
//! | transcript 进 -> 下一步文本出（无状态、可插拔） |  mock.rs /（llama.rs）
//! +----------------------------------------------+
//! ```
//!
//! 依赖方向自上而下、单向：harness 认识 agent-core；agent-core 只认识
//! `LlmBackend` 抽象，不关心背后是 mock 还是 llama；llm 层对上面两层一无所知。
//!
//! 运行：`cargo run -- --demo`

use crate::agent::{Agent, Turn};
use crate::backend::{LlmBackend, MockBackend};
use crate::config::AppConfig;
use crate::error::Result;
use crate::tools::ToolRegistry;

/// 演示入口（由 main.rs 的 `--demo` 触发）。
/// 注意：这个函数以及它做的每件事，本身就在扮演 harness 的角色。
pub fn run() -> Result<()> {
    banner("harness / agent-core / llm 三层演示");
    demo_llm_layer()?;
    demo_agent_core_layer()?;
    demo_harness_layer()?;
    summary();
    Ok(())
}

/// demo 1 · llm 层：后端只是「transcript 进 -> 下一步文本出」的映射。
/// 它不执行工具、也不知道循环的存在 —— 对比两次调用即可看清这一点。
fn demo_llm_layer() -> Result<()> {
    section("1/3 llm 层（src/backend/）：文本进 -> 文本出");
    let mut llm = MockBackend::new();

    // 第一次：transcript 里只有用户问题。
    let t1 = "system: …工具协议提示词…\nuser: 现在几点了?\n";
    let o1 = llm.generate(t1)?;
    println!("  喂入 transcript:");
    print_indented(t1);
    println!("  后端产出: {o1}");

    // 第二次：同一个问题，但 transcript 里多回填了「上一步 CALL + 工具观察」。
    let t2 = "system: …工具协议提示词…\n\
              user: 现在几点了?\n\
              assistant: CALL: get_time\n\
              tool: OBSERVATION: 当前 Unix 时间戳（秒）: 1789091041\n";
    let o2 = llm.generate(t2)?;
    println!("\n  再喂入（比上面多了 assistant / tool 两行）:");
    print_indented(t2);
    println!("  后端产出: {o2}");

    point(
        "llm 层没有循环、也不执行工具：它只根据「完整 transcript」决定下一步输出什么；\
         执行工具、来回循环的是上一层 —— agent-core。",
    );
    Ok(())
}

/// demo 2 · agent-core 层：Agent 拿着后端与工具，跑「模型 -> 工具 -> 观察 -> 模型 …」循环。
/// 它只依赖 `LlmBackend` 抽象且自身零 I/O —— 循环内部靠 harness 注入的 tracer 展示。
fn demo_agent_core_layer() -> Result<()> {
    section("2/3 agent-core 层（src/agent.rs + src/tools/）：推理循环");

    let mut agent = build_agent(Box::new(MockBackend::new()));
    agent.set_tracer(Box::new(|ev| println!("    [trace] {ev}")));

    let mut history: Vec<Turn> = Vec::new();
    println!("  第 1 轮提问: 「现在几点了?」");
    let ans = agent.run_turn(&mut history, "现在几点了?")?;
    println!("  最终答案: {ans}");

    println!("\n  第 2 轮提问: 「echo hello layers」（复用同一份 history，上下文保留）");
    let ans = agent.run_turn(&mut history, "echo hello layers")?;
    println!("  最终答案: {ans}");

    point(
        "agent-core 解析 CALL:/ANSWER: 协议、执行工具、把 OBSERVATION 回填进 transcript\
         再继续问后端，直到拿到最终答案；跨轮的 history 不断累积上下文。",
    );
    Ok(())
}

/// demo 3 · harness 层：组装 + 驱动。同一个问题、同一套循环，harness 换个后端（大脑）
/// 行为就完全不同 —— 而 agent-core 和工具层一行代码都没改。
fn demo_harness_layer() -> Result<()> {
    section("3/3 harness 层（src/main.rs + src/config.rs）：组装 + 驱动");
    println!("  同一个问题、同一套 agent-core 循环，harness 换个后端试试:\n");

    println!("  [后端 A] mock —— 脚本化大脑，会调用工具:");
    one_turn(build_agent(Box::new(MockBackend::new())), "现在几点了?")?;

    println!("\n  [后端 B] direct —— 演示用大脑，从不调用工具:");
    one_turn(build_agent(Box::new(DirectBackend)), "现在几点了?")?;

    point(
        "harness 决定「用哪个后端、注册哪些工具、怎么展示」（正是 main.rs 里那几步）；\
         agent-core 只认 LlmBackend 抽象 —— 这就是后端可插拔的原因。",
    );
    Ok(())
}

/// 演示用极简后端：从不调用工具，直接作答。
/// 存在意义只有一个：证明 agent-core 只认 `LlmBackend` 抽象，
/// harness 塞给它哪个「大脑」都能跑，循环逻辑零改动。
struct DirectBackend;

impl LlmBackend for DirectBackend {
    fn name(&self) -> &str {
        "direct"
    }

    fn generate(&mut self, transcript: &str) -> Result<String> {
        let user = transcript
            .lines()
            .rev()
            .find_map(|l| l.strip_prefix("user: "))
            .unwrap_or("(无用户输入)");
        Ok(format!("ANSWER: (direct) 我不使用工具，直接回答: {user}"))
    }
}

// ---------------------------------------------------------------------------
// 以下是本 demo 自己的「harness 式」组装 / 驱动小工具。
// ---------------------------------------------------------------------------

/// harness 式组装：后端 + 内置工具 + 默认配置 -> Agent。
fn build_agent(backend: Box<dyn LlmBackend>) -> Agent {
    let mut tools = ToolRegistry::new();
    tools.register_builtins();
    Agent::new(backend, tools, &AppConfig::default())
}

/// 装上 tracer、跑一轮、打印答案。
fn one_turn(mut agent: Agent, question: &str) -> Result<()> {
    agent.set_tracer(Box::new(|ev| println!("    [trace] {ev}")));
    let mut history: Vec<Turn> = Vec::new();
    let ans = agent.run_turn(&mut history, question)?;
    println!("    最终答案: {ans}");
    Ok(())
}

fn print_indented(t: &str) {
    for line in t.lines() {
        println!("      {line}");
    }
}

fn banner(title: &str) {
    println!("\n========== {title} ==========");
}

fn section(title: &str) {
    println!("\n---------- {title} ----------");
}

fn point(text: &str) {
    println!("\n  >> 要点: {text}");
}

fn summary() {
    println!("\n========== 总结 ==========");
    println!("  harness     组装与驱动（选后端、配工具、管 I/O）         main.rs / config.rs");
    println!("  agent-core  推理循环（协议解析、工具分发、历史管理）     agent.rs / tools/");
    println!("  llm         大脑（transcript 进 -> 下一步出，可插拔）    backend/");
    println!();
    println!("  依赖方向: harness -> agent-core -> llm，自上而下单向；");
    println!("  下层不知道上层的存在（mock 后端并不知道自己被谁驱动）。");
}
