//! Opinionated wrapper around subprocess calls so they can be handeled on the fly.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub enum ProcessingResult {
    Warning(String),
    Content(String),
    Silent,
}

#[derive(Default)]
pub struct HandledCmdOut {
    pub warnings: Vec<String>,
    pub content: Vec<String>,
}

pub trait Handler {
    fn process(&mut self, value: &String) -> Result<ProcessingResult, String>;
}

/// Holds the command arguments and the handlers that will be ran on each output in real time.
pub struct HandledCommand {
    // The Command instance that will be executed in a piped subprocess.
    command: Command,
    // Controls wether the output line is supposed to be returned and in which bin (warning or content).
    // In case no handler is defined, everythning is consdiered content.
    handler: Option<Box<dyn Handler>>,
    input: Option<String>,
}

impl HandledCommand {
    /// Creates new command
    pub fn new(command: impl AsRef<OsStr>) -> Self {
        Self {
            command: Command::new(command),
            handler: None,
            input: None,
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

    pub fn set_handler(mut self, handler: Box<dyn Handler>) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn set_input(mut self, input: String) -> Self {
        self.input = Some(input);
        self
    }

    fn send_input(&mut self) -> Result<HandledCmdOut, String> {
        let pipe_result = Command::new("p4")
            .arg("change")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = match pipe_result {
            Err(e) => return Err(format!("Failed to execute command: {}", e)),
            Ok(pipe) => pipe,
        };

        let input = self.input.clone().unwrap();
        {
            let stdin = pipe.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(input.as_bytes()).unwrap();
            stdin.flush().unwrap();
        }

        let output = pipe.wait_with_output().unwrap();
        if output.status.success() {
            let std_out = String::from_utf8_lossy(&output.stdout);
            let mut result = HandledCmdOut::default();
            result.content.push(std_out.into_owned());
            return Ok(result);
        } else {
            let std_err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to write stdin: {}", std_err.to_owned()));
        }
    }

    pub fn execute(&mut self) -> Result<HandledCmdOut, String> {
        if self.input.is_some() {
            return self.send_input();
        }

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

        let mut result = HandledCmdOut::default();

        for line in BufReader::new(stdout_handler)
            .lines()
            .filter_map(|line| line.ok())
        {
            let Some(handler) = &mut self.handler else {
                // In case of no handler, everything is considered a content.
                result.content.push(line);
                continue;
            };

            match handler.process(&line) {
                Err(e) => return Err(format!("Failed to handle output: {}", e)),
                Ok(ProcessingResult::Warning(v)) => result.warnings.push(v),
                Ok(ProcessingResult::Content(v)) => result.content.push(v),
                _ => {}
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
