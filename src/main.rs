use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use rustyline::Editor;
use rustyline::config::Config as RustylineConfig;

struct LinuxCommandAssistant {
    config: Config,
    context: Vec<Message>,
    recent_interactions: VecDeque<String>,
    is_ai_mode: bool,
}

impl LinuxCommandAssistant {
    fn new(config: Config) -> Self {
        Self {
            config,
            context: Vec::new(),
            recent_interactions: VecDeque::new(),
            is_ai_mode: false,
        }
    }

  async fn run(&mut self) -> Result<()> {
        let config = RustylineConfig::builder()
            .history_ignore_space(true)
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(LinuxCommandCompleter));

        loop {
            let prompt = if self.is_command_mode { 
                format!("{}$ {}", BLUE, RESET) 
            } else { 
                format!("{}kaka-ai> {}", YELLOW, RESET) 
            };
            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.eq_ignore_ascii_case("exit") {
                        break;
                    }

                    if line.eq_ignore_ascii_case("reset") {
                        self.context.clear();
                        self.recent_interactions.clear();
                        println!("Context and recent interactions have been reset.");
                        continue;
                    }

                    if !line.is_empty() && !line.starts_with('#') {
                        self.add_to_history(line.to_string());
                        rl.add_history_entry(line);
                    }

                    if line == "!" {
                        self.is_command_mode = !self.is_command_mode;
                        if self.is_command_mode {
                            println!("Entered Linux command mode. Type '!' to exit.");
                        } else {
                            println!("Exited Linux command mode.");
                        }
                        continue;
                    }

                    if self.is_command_mode {
                        match self.execute_command(line) {
                            Ok(_) => (), // 命令已经直接执行，输出已经显示在终端上
                            Err(e) => println!("Error executing command: {}", e),
                        }
                    } else {
                        match self.get_ai_response(line).await {
                            Ok(response) => {
                                println!("kaka-AI: {}", response);
                                self.update_context(line, &response);
                                self.add_to_recent_interactions(format!("User: {}\nAI: {}", line, response));
                            }
                            Err(e) => println!("Error getting AI response: {}", e),
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, command: &str) -> Result<()> {
    // 直接执行命令，不做任何封装
    std::process::Command::new(command)
        .status()
        .context("Failed to execute command")?;

    // 只记录执行的命令
    self.add_to_recent_interactions(format!("Executed command: {}", command));

    Ok(())
}


    fn capture_command_output(&mut self, command: &str) -> Result<()> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute command")?;

        let mut output = String::new();
        if let Some(stdout) = child.stdout.take() {
            io::copy(&mut stdout, &mut io::stdout())?;
            io::copy(&mut stdout, &mut output)?;
        }
        if let Some(stderr) = child.stderr.take() {
            io::copy(&mut stderr, &mut io::stderr())?;
            io::copy(&mut stderr, &mut output)?;
        }

        let status = child.wait()?;

        // 记录命令和输出
        if !self.is_long_running_command(command) {
            self.add_to_recent_interactions(format!("$ {}\n{}", command, output));
        }

        if !status.success() {
            println!("Command failed with exit code: {:?}", status.code());
        }

        Ok(())
    }

    fn is_long_running_command(&self, command: &str) -> bool {
        command.starts_with("top") || command.starts_with("vim") || command.starts_with("nano")
    }

    fn add_to_recent_interactions(&mut self, interaction: String) {
        self.recent_interactions.push_back(interaction);
        if self.recent_interactions.len() > self.config.max_recent_interactions {
            self.recent_interactions.pop_front();
        }
    }

    async fn handle_ai_interaction(&mut self, input: &str) -> Result<()> {
        // 实现与 OpenAI 交互的逻辑
        // ...
        Ok(())
    }
}
