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

        loop {
            let prompt = if self.is_ai_mode {
                "kaka-ai> "
            } else {
                "$ "
            };
            let readline = rl.readline(prompt);

            match readline {
                Ok(line) => {
                    if line.trim() == "exit" {
                        break;
                    }

                    if line.trim() == "!" {
                        self.is_ai_mode = !self.is_ai_mode;
                        println!("{}", if self.is_ai_mode {
                            "Entered AI mode. Type '!' to exit."
                        } else {
                            "Exited AI mode."
                        });
                        continue;
                    }

                    if !self.is_ai_mode {
                        self.capture_command_output(&line)?;
                    } else {
                        self.handle_ai_interaction(&line).await?;
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
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
