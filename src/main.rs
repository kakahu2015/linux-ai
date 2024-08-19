use std::io::{self, Write};
use std::collections::VecDeque;
use std::process::{Command, Stdio};

const MAX_ENTRIES: usize = 10;

fn main() -> io::Result<()> {
    let mut history = VecDeque::with_capacity(MAX_ENTRIES);
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush()?;

        input.clear();
        if io::stdin().read_line(&mut input)? == 0 {
            break; // EOF
        }

        let command = input.trim();
        if !command.is_empty() {
            // 添加命令到历史
            if history.len() >= MAX_ENTRIES {
                history.pop_front();
            }
            history.push_back(command.to_string());

            // 检查是否是退出命令
            if command == "quit" {
                println!("Exiting...");
                break;
            }

            // 执行命令
            let mut parts = command.split_whitespace();
            let program = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.collect();

            let status = if program == "top" || program == "vim" || program == "nano" {
                // 对于交互式命令，使用不同的执行方式
                Command::new(program)
                    .args(&args)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()?
            } else {
                // 对于其他命令，使用原来的执行方式
                Command::new(program)
                    .args(&args)
                    .status()?
            };

            if !status.success() {
                eprintln!("Command failed: {}", command);
            }
        }
    }

    Ok(())
}
