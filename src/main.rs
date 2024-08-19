use std::io::{self, Write};
use std::collections::VecDeque;
use std::process::Command;
use std::io::Read;

const MAX_ENTRIES: usize = 10;

fn main() -> io::Result<()> {
    let mut history = VecDeque::with_capacity(MAX_ENTRIES);
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush()?;

        // 使用 `read` 方法读取用户输入，并检查是否为 `Ctrl + q`
        let mut buffer = [0; 1];
        if io::stdin().read(&mut buffer)? == 0 || (buffer[0] == 3 && input.is_empty()) {
            break; // EOF or Ctrl + q
        }

        // 将用户输入添加到 `input` 字符串
        input.push(buffer[0] as char);

        // 如果用户输入回车，则处理命令
        if input.ends_with('\n') {
            let command = input.trim();
            if !command.is_empty() {
                // 添加命令到历史
                if history.len() >= MAX_ENTRIES {
                    history.pop_front();
                }
                history.push_back(command.to_string());

                // 执行命令
                let status = Command::new(command.split_whitespace().next().unwrap_or(""))
                    .args(command.split_whitespace().skip(1))
                    .status();

                match status {
                    Ok(status) => {
                        if !status.success() {
                            eprintln!("Command failed: {}", command);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error executing command: {}", e);
                    }
                }
            }
            input.clear(); // 清空输入缓冲区
        }
    }

    Ok(())
}
