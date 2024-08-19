use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

const MAX_ENTRIES: usize = 10;

#[derive(Clone, Debug)]
struct CommandEntry {
    input: String,
    output: String,
}

#[tokio::main]
async fn main() {
    let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_ENTRIES)));
    
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();
    let mut input = String::new();

    loop {
        input.clear();
        writer.write_all(b"$ ").await.unwrap();
        writer.flush().await.unwrap();

        if reader.read_line(&mut input).await.unwrap() == 0 {
            break;
        }

        let input = input.trim().to_string();
        let output = format!("Output for: {}", input);

        let mut buffer = buffer.lock().await;
        buffer.push_back(CommandEntry { input, output: output.clone() });
        if buffer.len() > MAX_ENTRIES {
            buffer.pop_front();
        }

        writer.write_all(output.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();

        // 打印当前缓冲区内容
        println!("Current buffer:");
        for (i, entry) in buffer.iter().enumerate() {
            println!("{}: {:?}", i, entry);
        }
    }
}
