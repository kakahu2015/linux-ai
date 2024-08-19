use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::process::Command;
use std::time::{Duration, SystemTime};
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,

    #[arg(short, long)]
    command: String,

    #[arg(long, default_value = "10485760")] // 默认10MB
    max_cache_size: usize,

    #[arg(long, default_value = "3600")] // 默认1小时
    max_cache_age: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    record_commands: HashSet<String>,
}

struct CommandRecord {
    command: String,
    stdout: String,
    stderr: String,
    timestamp: SystemTime,
    size: usize,
}

struct MemoryCache {
    records: VecDeque<CommandRecord>,
    max_size_bytes: usize,
    current_size_bytes: usize,
    max_age: Duration,
}

impl CommandRecord {
    fn new(command: String, stdout: String, stderr: String) -> Self {
        let size = command.len() + stdout.len() + stderr.len();
        Self {
            command,
            stdout,
            stderr,
            timestamp: SystemTime::now(),
            size,
        }
    }
}

impl MemoryCache {
    fn new(max_size_bytes: usize, max_age: Duration) -> Self {
        Self {
            records: VecDeque::new(),
            max_size_bytes,
            current_size_bytes: 0,
            max_age,
        }
    }

    fn add(&mut self, record: CommandRecord) {
        // 移除过旧的记录
        let now = SystemTime::now();
        while let Some(front) = self.records.front() {
            if now.duration_since(front.timestamp).unwrap() > self.max_age {
                let removed = self.records.pop_front().unwrap();
                self.current_size_bytes -= removed.size;
            } else {
                break;
            }
        }

        // 移除记录直到有足够的空间
        while self.current_size_bytes + record.size > self.max_size_bytes && !self.records.is_empty() {
            let removed = self.records.pop_front().unwrap();
            self.current_size_bytes -= removed.size;
        }

        // 添加新记录
        self.current_size_bytes += record.size;
        self.records.push_back(record);
    }

    fn get_context(&self) -> String {
        self.records
            .iter()
            .map(|r| format!(
                "Command: {}\nTimestamp: {:?}\nStdout: {}\nStderr: {}\n",
                r.command,
                r.timestamp,
                r.stdout,
                r.stderr
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }
}


async fn interact_with_openai(client: &Client, context: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant analyzing command line output."},
                {"role": "user", "content": format!("Analyze the following command outputs:\n\n{}", context)}
            ]
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = std::fs::read_to_string(&args.config)?;
    let config: Config = serde_yaml::from_str(&config)?;

    let mut cache = MemoryCache::new(
        args.max_cache_size,
        Duration::from_secs(args.max_cache_age)
    );
    let client = Client::new();

    if config.record_commands.contains(&args.command) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        println!("Command: {}", args.command);
        println!("Stdout: {}", stdout);
        println!("Stderr: {}", stderr);

        cache.add(CommandRecord::new(args.command, stdout, stderr));

        // 与OpenAI API交互
        let context = cache.get_context();
        let analysis = interact_with_openai(&client, &context).await?;
        println!("\nOpenAI Analysis:\n{}", analysis);
    } else {
        println!("Command not in record list. Executing without recording.");
        let status = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
