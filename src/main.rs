use std::io::{self, Write};
use std::collections::VecDeque;
use std::process::Command;

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

            // 直接执行命令
            let status = Command::new(command.split_whitespace().next().unwrap_or(""))
                .args(command.split_whitespace().skip(1))
                .status()?;

            if !status.success() {
                eprintln!("Command failed: {}", command);
            }
        }
    }

    Ok(())
}
