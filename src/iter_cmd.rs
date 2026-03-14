///! Thin wrapper around std::process that makes subprocess call execute as iterators.
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Lines, Write};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

use crate::error::P4OxiError;

/// Holds the command arguments and the handlers that will be ran on each output in real time.
pub struct IterCmd {
    // The Command instance that will be executed in a piped subprocess.
    command: Command,
    // If set than this will be sent to stdin when send or execute is called.
    input: Option<String>,
}

pub struct RunningIterCmd {
    pipe: Child,
    stdout: Lines<BufReader<ChildStdout>>,
    stderr: Lines<BufReader<ChildStderr>>,
    end_reached: bool,
}

impl IterCmd {
    /// Creates new command
    pub fn new(command: impl AsRef<OsStr>) -> Self {
        Self {
            command: Command::new(command),
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

    pub fn set_input(mut self, input: String) -> Self {
        self.input = Some(input);
        self
    }

    /// Will send the input to the subprocess (used for sending Specs)
    pub fn send_input(&mut self) -> Result<String, P4OxiError> {
        let pipe_result = Command::new("p4")
            .arg("change")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = match pipe_result {
            Err(e) => {
                return Err(P4OxiError::ProcessError(format!(
                    "Failed to execute command: {e}"
                )));
            }
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
            Ok(std_out.into_owned())
        } else {
            let std_err = String::from_utf8_lossy(&output.stderr);
            Err(P4OxiError::ProcessError(format!(
                "Failed to write stdin: {std_err}"
            )))
        }
    }

    /// Starts the command and returns an iterator that can be lazily evaluated.
    pub fn run(&mut self) -> Result<RunningIterCmd, P4OxiError> {
        if self.input.is_some() {
            return Err(P4OxiError::InvalidUsage(
                "Command has an input, it cannot be run as iterator! Use .send_input instead.",
            ));
        }

        let pipe_result = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = match pipe_result {
            Err(e) => {
                return Err(P4OxiError::ProcessError(format!(
                    "Failed to spawn process: {e}"
                )));
            }
            Ok(pipe) => pipe,
        };

        let Some(stdout_handler) = pipe.stdout.take() else {
            return Err(P4OxiError::ProcessError(
                "Failed to capture stdout".to_string(),
            ));
        };

        let Some(stderr_handler) = pipe.stderr.take() else {
            return Err(P4OxiError::ProcessError(
                "Failed to capture stderr".to_string(),
            ));
        };

        Ok(RunningIterCmd {
            pipe: pipe,
            stdout: BufReader::new(stdout_handler).lines(),
            stderr: BufReader::new(stderr_handler).lines(),
            end_reached: false,
        })
    }
}

impl RunningIterCmd {
    pub fn wait(&mut self) -> Result<(), P4OxiError> {
        let Ok(exit_status) = self.pipe.wait() else {
            return Err(P4OxiError::ProcessError(
                "Failed to wait on process (no exit_status received)".to_string(),
            ));
        };

        if !exit_status.success() {
            Err(P4OxiError::ProcessError(format!(
                "Process exited with status: {exit_status:?}"
            )))
        } else {
            Ok(())
        }
    }
}

impl Iterator for RunningIterCmd {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Ok(line)) = self.stdout.next() {
            Some(Ok(line))
        } else if let Some(Ok(line)) = self.stderr.next() {
            Some(Err(line))
        } else if !self.end_reached {
            self.end_reached = true;
            match self.wait() {
                Ok(()) => None,
                Err(err) => Some(Err(err.to_string())),
            }
        } else {
            None
        }
    }
}

impl Drop for RunningIterCmd {
    fn drop(&mut self) {
        let _ = self.pipe.kill();
    }
}
