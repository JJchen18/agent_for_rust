mod agent;
mod backend;
mod config;
mod error;
mod tools;

use agent::{Agent, Turn};
use backend::build_backend;
use config::AppConfig;
use error::{AgentError, Result};
use std::io::{self, BufRead, Write};
use tools::ToolRegistry;

fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = parse_args(&args)?;

    let mut tools = ToolRegistry::new();
    tools.register_builtins();

    let backend = build_backend(&cfg)?;
    let mut agent = Agent::new(backend, tools, &cfg);

    // 一次性模式：--ask "<问题>"
    if let Some(question) = &cfg.ask {
        let mut history: Vec<Turn> = Vec::new();
        let answer = agent.run_turn(&mut history, question)?;
        println!("{answer}");
        return Ok(());
    }

    // 交互式 REPL。
    println!(
        "agent 就绪（后端: {}）。输入问题开始对话，输入 quit 退出。",
        agent.backend_name()
    );
    let stdin = io::stdin();
    let mut history: Vec<Turn> = Vec::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break; // EOF
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        match agent.run_turn(&mut history, line) {
            Ok(answer) => println!("{answer}"),
            Err(e) => eprintln!("错误: {e}"),
        }
    }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<AppConfig> {
    let mut cfg = AppConfig::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--backend" => {
                let v = value_of(args, &mut i, "--backend")?;
                let kind: config::BackendKind = v
                    .parse()
                    .map_err(|e: String| AgentError::Config(e))?;
                cfg.backend = kind;
            }
            "--model" => cfg.model_path = Some(value_of(args, &mut i, "--model")?),
            "--system" => cfg.system = Some(value_of(args, &mut i, "--system")?),
            "--max-iters" => {
                let v = value_of(args, &mut i, "--max-iters")?;
                cfg.max_iters = v
                    .parse()
                    .map_err(|_| AgentError::Config(format!("--max-iters: {v:?} 不是数字")))?;
            }
            "--ask" => cfg.ask = Some(value_of(args, &mut i, "--ask")?),
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(AgentError::Config(format!(
                    "未知参数 {other:?}（见 --help）"
                )))
            }
        }
    }
    Ok(cfg)
}

/// args[*i] 是某个 flag，返回值是它后面的值，并把 *i 前进到该值之后。
fn value_of(args: &[String], i: &mut usize, flag: &str) -> Result<String> {
    let v = args.get(*i + 1).cloned().ok_or_else(|| {
        AgentError::Config(format!("{flag} 需要一个值"))
    })?;
    *i += 2; // 跳过 flag 与它的值
    Ok(v)
}

fn print_help() {
    println!(
"agent —— 端侧 agent 骨架（纯 std，零外部依赖）

用法: agent_test [选项]
  --backend <mock|llama>   推理后端（默认 mock）
  --model <path.gguf>      GGUF 模型路径（llama 后端使用）
  --system <文本>          自定义系统提示词
  --max-iters <n>          每轮最多工具调用迭代次数（默认 5）
  --ask <问题>             一次性：回答后直接退出（否则进入 REPL）
  -h, --help               显示本帮助

示例:
  agent_test                                    # 用 mock 后端进入 REPL
  agent_test --ask \"现在几点了?\"              # 一次性（mock 会调用 get_time）
  agent_test --backend llama --model m.gguf    # 待 llama 后端启用后
"
    );
}
