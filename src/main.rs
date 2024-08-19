use std::sync::Arc;
use std::io::{self, BufRead, Read};
use std::thread;
use std::env;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use reqwest;

#[derive(Clone, Serialize, Deserialize)]
struct CommandRecord {
    input: String,
    output: String,
}

type SharedMemory = Arc<Mutex<Vec<CommandRecord>>>;

fn listener_thread(shared_memory: SharedMemory) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let mut output = String::new();
        stdout.read_to_string(&mut output).unwrap();

        let mut memory = shared_memory.lock();
        memory.push(CommandRecord { input, output });
        if memory.len() > 10 {
            memory.remove(0);
        }
    }
}

async fn ai_dialog(shared_memory: SharedMemory) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // 从环境变量读取 API 密钥
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    
    loop {
        print!("AI> ");
        io::stdout().flush()?;
        
        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        
        if query.trim() == "exit" {
            break;
        }
        
        let memory = shared_memory.lock();
        let context: Vec<CommandRecord> = memory.clone();
        
        let prompt = format!(
            "Recent commands and outputs:\n{}\n\nUser query: {}",
            context.iter()
                .map(|record| format!("Command: {}\nOutput: {}", record.input, record.output))
                .collect::<Vec<_>>()
                .join("\n\n"),
            query.trim()
        );
        
        let response = client
            .post("https://api.openai.com/v1/engines/davinci-codex/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "prompt": prompt,
                "max_tokens": 150
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        println!("AI: {}", response["choices"][0]["text"].as_str().unwrap());
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_memory: SharedMemory = Arc::new(Mutex::new(Vec::new()));
    
    let listener_memory = shared_memory.clone();
    thread::spawn(move || {
        listener_thread(listener_memory);
    });

    tokio::runtime::Runtime::new()?.block_on(ai_dialog(shared_memory))?;

    Ok(())
}
