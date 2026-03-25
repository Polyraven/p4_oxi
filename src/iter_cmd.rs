//! Thin wrapper around std::process that makes subprocess call execute as iterators.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Lines, Write};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

use crate::error::P4Error;

/// Holds the command arguments and spawns a subprocess that can be iterated over.
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

    /// Will send the input to the subprocess (used for sending Forms)
    pub fn send_input(&mut self) -> Result<String, P4Error> {
        let pipe_result = self
            .command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = pipe_result
            .map_err(|e| P4Error::ProcessError("Failed to spawn process".to_string(), Some(e)))?;

        let Some(input) = self.input.clone() else {
            return Err(P4Error::InvalidUsage(
                ".send_input() called with no input actually set.",
            ));
        };
        let stdin = pipe
            .stdin
            .as_mut()
            .ok_or_else(|| P4Error::ProcessError("Failed to open stdin".to_string(), None))?;
        stdin.write_all(input.as_bytes())?;
        stdin.flush()?;

        let output = pipe.wait_with_output()?;
        if output.status.success() {
            let std_out = String::from_utf8_lossy(&output.stdout);
            Ok(std_out.into_owned())
        } else {
            let std_err = String::from_utf8_lossy(&output.stderr);
            Err(P4Error::ProcessError(format!("stderr: {std_err}"), None))
        }
    }

    /// Starts the command and returns an iterator that can be lazily evaluated.
    pub fn run(&mut self) -> Result<RunningIterCmd, P4Error> {
        if self.input.is_some() {
            return Err(P4Error::InvalidUsage(
                "Command has an input, it cannot be run as iterator! Use .send_input instead.",
            ));
        }

        let pipe_result = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut pipe = pipe_result
            .map_err(|e| P4Error::ProcessError("Failed to spawn process".to_string(), Some(e)))?;

        let Some(stdout_handler) = pipe.stdout.take() else {
            return Err(P4Error::ProcessError(
                "Failed to capture stdout".to_string(),
                None,
            ));
        };

        let Some(stderr_handler) = pipe.stderr.take() else {
            return Err(P4Error::ProcessError(
                "Failed to capture stderr".to_string(),
                None,
            ));
        };

        Ok(RunningIterCmd {
            pipe,
            stdout: BufReader::new(stdout_handler).lines(),
            stderr: BufReader::new(stderr_handler).lines(),
            end_reached: false,
        })
    }
}

impl RunningIterCmd {
    pub fn wait(&mut self) -> Result<(), P4Error> {
        let exit_status = self
            .pipe
            .wait()
            .map_err(|e| P4Error::ProcessError("Failed to wait on process".to_string(), Some(e)))?;

        if !exit_status.success() {
            Err(P4Error::ProcessError(
                format!("Process exited with status: {exit_status:?}"),
                None,
            ))
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
        let _ = self.pipe.wait();
    }
}
