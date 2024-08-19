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
        if command.is_empty() {
            continue;
        }

        if command == "quit" {
            break; // 退出程序
        }

        // 添加命令到历史
        if history.len() >= MAX_ENTRIES {
            history.pop_front();
        }
        history.push_back(command.to_string());

        // 执行命令
        let mut parts = command.split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        let status = Command::new(command)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            eprintln!("Command failed with exit code: {:?}", status.code());
        }
    }

    Ok(())
}
