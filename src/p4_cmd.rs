//! General wrapper around P4 CLI commands.

use crate::{
    global_options::GlobalOptions,
    handled_command::HandledCommand,
    handlers::{InfoHandler, OutHandler, P4CmdHandlers, WarningHandler},
    iter_cmd::IterCmd,
    p4_running_cmd::P4RunningCmd,
    warnings::P4Message,
};

/// A General low-level wrapper around P4 CLI commands.
///
/// While to the CLI we would send one line full with arguments,
/// This here splits up those to logical pieces.
/// Other methods in this library use this struct for executing their commands.
///
/// The handlers used in here are also going to be called during command execution, not after.
///
/// Example:
/// ```ignore
///     P4Cmd::new("fstat")
///     .wildcard(r"c:\my_workspace\my_directory\...")
///     .execute()
///     .unwrap();
///```
///
/// CmdRef documentation:
/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html
#[derive(Default)]
pub struct P4Cmd {
    pub g_option: GlobalOptions,
    /// The P4 command that will be executed, like fstat, sync, clean etc.
    pub cmd: String,
    /// For the arguments that start with a `-`
    pub args: Vec<String>,
    /// Last argument, needs special validation
    pub wildcards: Vec<String>,
    /// All output message will go throug this
    pub handler: P4CmdHandlers,
    /// Used when creating a changelist or a stream etc. by feeding the bytes in with stdin.
    pub input_sepcification: Option<Box<dyn P4Specification>>,
}

impl P4Cmd {
    pub fn raw(cmd: &str) -> Self {
        P4Cmd {
            cmd: cmd.to_string(),
            ..Default::default()
        }
    }

    pub fn with_g_options(mut self, g_options: GlobalOptions) -> Self {
        self.g_option = g_options;
        self
    }

    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn with_arg_value(mut self, arg: &str, value: String) -> Self {
        self.args.push(arg.to_string());
        self.args.push(value);
        self
    }

    pub fn with_args(mut self, args: Vec<&str>) -> Self {
        args.iter().for_each(|arg| self.args.push(arg.to_string()));
        self
    }

    pub fn with_wildcard(mut self, wildcard: &str) -> Self {
        self.wildcards.push(wildcard.to_string());
        self
    }

    pub fn with_wildcards(mut self, wildcards: Vec<&str>) -> Self {
        wildcards
            .iter()
            .for_each(|wildcard| self.wildcards.push(wildcard.to_string()));
        self
    }

    pub fn warning_handler(mut self, handler: Box<dyn WarningHandler>) -> Self {
        self.handler.warning_handlers.push(handler);
        self
    }

    pub fn info_handler(mut self, handler: Box<dyn InfoHandler>) -> Self {
        self.handler.info_handlers.push(handler);
        self
    }

    pub fn with_output_handler(mut self, handler: Box<dyn OutHandler>) -> Self {
        self.handler.output_handlers.push(handler);
        self
    }

    pub fn with_input_spec(mut self, input_spec: Box<dyn P4Specification>) -> Self {
        self.g_option.format_as_json = false;
        self.input_sepcification = Some(input_spec);
        self
    }

    pub fn run<T>(self) -> Result<P4RunningCmd<T>, String> {
        if self.input_sepcification.is_some() {
            return Err(
                "A command cannot be called when input_spec is set (unintended missue? For uploading specs call .submit_spec())".to_string(),
            );
        }

        let iter_cmd = IterCmd::from(self).run()?;
        Ok(P4RunningCmd::new(iter_cmd))
    }

    pub fn execute<T>(self) -> Result<Vec<Result<T, P4Message>>, String> {
        self.run::<T>()?.collect()
    }

    pub fn submit_spec(self) -> Result<String, String> {
        // returns the raw data for now, but the Specification should implement what this returns.
        // Also note thet send_input will use change now right away.
        IterCmd::from(self).send_input()
    }
}

impl From<P4Cmd> for HandledCommand {
    fn from(p4_cmd: P4Cmd) -> HandledCommand {
        let mut cmd = HandledCommand::new("p4")
            .args(&p4_cmd.g_option.to_arg_list())
            .arg(&p4_cmd.cmd)
            .args(&p4_cmd.args)
            .args(&p4_cmd.wildcards)
            .set_handler(Box::new(p4_cmd.handler));

        if p4_cmd.input_sepcification.is_some() {
            cmd = cmd.set_input(p4_cmd.input_sepcification.unwrap().convert());
        }

        cmd
    }
}

impl From<P4Cmd> for IterCmd {
    fn from(p4_cmd: P4Cmd) -> Self {
        let mut cmd = IterCmd::new("p4")
            .args(&p4_cmd.g_option.to_arg_list())
            .arg(&p4_cmd.cmd)
            .args(&p4_cmd.args)
            .args(&p4_cmd.wildcards);

        if let Some(input_spec) = p4_cmd.input_sepcification {
            cmd = cmd.set_input(input_spec.convert());
        }
        cmd
    }
}

pub trait P4CmdArgs<T> {
    fn to_p4_cmd(&self) -> P4Cmd;

    fn run(&self) -> Result<P4RunningCmd<T>, String> {
        self.to_p4_cmd().run::<T>()
    }

    fn execute(&self) -> Result<Vec<Result<T, P4Message>>, String> {
        self.to_p4_cmd().execute::<T>()
    }

    fn submit_spec(&self) -> Result<String, String> {
        self.to_p4_cmd().submit_spec()
    }
}

pub trait P4Specification {
    fn convert(&self) -> String;
}
