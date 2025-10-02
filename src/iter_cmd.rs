use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Lines, Write};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

pub enum ProcessingResult {
    Warning(String),
    Content(String),
    Silent,
}

/// Holds the command arguments and the handlers that will be ran on each output in real time.
pub struct IterCmd {
    // The Command instance that will be executed in a piped subprocess.
    command: Command,
    // If set than this will be sent to stdin when send or execute is called.
    input: Option<String>,
}

#[derive(Default)]
pub struct CmdOut {
    pub warning: String,
    pub content: String,
}

pub trait Handler {
    fn process(&mut self, value: &String) -> Result<ProcessingResult, String>;
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

    /// Will send the input to the subprocess (used for sending Specs)
    fn send_input(&mut self) -> Result<String, String> {
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
            return Ok(std_out.into_owned());
        } else {
            let std_err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to write stdin: {}", std_err.to_owned()));
        }
    }

    /// Starts the command and returns an iterator that can be lazily evaluated.
    pub fn run(&mut self) -> Result<RunningIterCmd, String> {
        if self.input.is_some(){
            return Err("Command has an input, it cannot be run as iterator!".to_string());
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

        let Some(stderr_handler) = pipe.stderr.take() else {
            return Err("Failed to capture stderr".to_string());
        };

        Ok(RunningIterCmd {
            pipe: pipe,
            stdout: BufReader::new(stdout_handler).lines(),
            stderr: BufReader::new(stderr_handler).lines(),
            end_reached: false,
        })
    }

    /// Evaluates the command in one go and returns the collected output.
    pub fn execute(&mut self) -> Result<Vec<String>, String> {
        if self.input.is_some() {
            match self.send_input() {
                Err(err) => Err(err),
                Ok(v) => Ok(vec![v]),
            }
        } else {
            self.run()?.collect()
        }
    }

}

impl RunningIterCmd {
    pub fn wait(&mut self) -> Result<(), String> {
        let Ok(exit_status) = self.pipe.wait() else {
            return Err("Failed to wait on process (no exit_status received)".to_string());
        };

        if !exit_status.success() {
            return Err(format!("Process exited with status: {:?}", exit_status));
        } else {
            return Ok(());
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
                Err(err) => Some(Err(err)),
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
