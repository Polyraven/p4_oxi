use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub enum ProcessingResult {
    Warning(String),
    Content(String),
    Silent,
}

#[derive(Default)]
pub struct HandledCommandResults {
    pub warnings: Vec<String>,
    pub content: Vec<String>,
}

pub trait Handler {
    fn process(&mut self, value: &String) -> Result<ProcessingResult, String>;
}

/// Holds the command arguments and the handlers that will be ran on each output in real time.
pub struct HandledCommand {
    command: Command,
    handlers: Vec<Box<dyn Handler>>,
}

impl HandledCommand {
    /// Creates new command
    pub fn new(command: impl AsRef<OsStr>) -> Self {
        Self {
            command: Command::new(command),
            handlers: vec![],
        }
    }

    /// Adds the arg to the command
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.command.arg(arg);
        self
    }

    /// Appends arguments to the command
    pub fn args(mut self, args: &[impl AsRef<OsStr>]) -> Self {
        self.command.args(args);
        self
    }

    pub fn handler(mut self, handler: Box<dyn Handler>) -> Self {
        self.handlers.push(handler);
        self
    }

    pub fn handlers(mut self, handlers: &mut Vec<Box<dyn Handler>>) -> Self {
        self.handlers.append(handlers);
        self
    }

    pub fn execute(&mut self) -> Result<HandledCommandResults, String> {
        let pipe_result = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = match pipe_result {
            Err(e) => return Err(format!("Failed to execute command: {}", e)),
            Ok(pipe) => pipe,
        };

        let Some(stdout_handler) = pipe.stdout.take() else {
            return Err("Failed to capture stdout!".to_string());
        };

        let mut result = HandledCommandResults::default();

        for line in BufReader::new(stdout_handler)
            .lines()
            .filter_map(|line| line.ok())
        {
            for handler in &mut self.handlers {
                // TODO: single handler to avoid duplication?
                // The multiple handlers is much more important for P4V
                match handler.process(&line) {
                    Err(e) => return Err(format!("Failed to handle output: {}", e)),
                    Ok(ProcessingResult::Warning(v)) => result.warnings.push(v),
                    Ok(ProcessingResult::Content(v)) => result.content.push(v),
                    _ => {}
                }
            }
        }

        let Some(stderr_handler) = pipe.stderr.take() else {
            return Err("Failed to capture stderr".to_string());
        };
        let error_string = BufReader::new(stderr_handler)
            .lines()
            .filter_map(|line| line.ok())
            .collect::<Vec<String>>()
            .join("\n");

        let Ok(exit_status) = pipe.wait() else {
            return Err("Failed to wait on process (no exit_status received)".to_string());
        };

        if !exit_status.success() || !error_string.is_empty() {
            return Err(format!(
                "Process exited with status: {:?}\nstderr:\n{}",
                exit_status, error_string
            ));
        } else {
            return Ok(result);
        }
    }
}
