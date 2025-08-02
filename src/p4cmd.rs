use serde::Deserialize;

use crate::{
    global_options::GlobalOptions,
    handled_command::{HandledCommand, Handler},
    warnings::{InfoMessage, WarningMessage},
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
#[derive(Debug, Default)]
pub struct P4Cmd {
    pub g_optipn: GlobalOptions,
    /// The P4 command that will be executed, like fstat, sync, clean etc.
    pub cmd: String,
    /// For the arguments that start with a `-`
    pub args: Vec<String>,
    /// Last argument, needs special validation
    pub wildcards: Vec<String>,
    /// All output message will go throug this
    pub handler: P4CmdHandler,
}

impl P4Cmd {
    pub fn new(cmd: &str) -> Self {
        P4Cmd {
            cmd: cmd.to_string(),
            ..Default::default()
        }
    }

    pub fn g_options(mut self, g_options: GlobalOptions) -> Self {
        self.g_optipn = g_options;
        self
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args(mut self, args: Vec<&str>) -> Self {
        args.iter().for_each(|arg| self.args.push(arg.to_string()));
        self
    }

    pub fn wildcard(mut self, wildcard: &str) -> Self {
        self.wildcards.push(wildcard.to_string());
        self
    }

    pub fn wildcards(mut self, wildcards: Vec<&str>) -> Self {
        wildcards
            .iter()
            .for_each(|wildcard| self.wildcards.push(wildcard.to_string()));
        self
    }
}

impl From<P4Cmd> for HandledCommand {
    fn from(p4_cmd: P4Cmd) -> HandledCommand {
        HandledCommand::new("p4")
            .args(&p4_cmd.g_optipn.to_arg_list())
            .arg(&p4_cmd.cmd)
            .args(&p4_cmd.args)
            .args(&p4_cmd.wildcards)
            .handler(Box::new(p4_cmd.handler))
    }
}

pub trait P4Execute {
    fn execute(self) -> Result<(), String>;
}

impl P4Execute for P4Cmd {
    fn execute(self) -> Result<(), String> {
        HandledCommand::from(self).execute()
    }
}

#[derive(Debug, Default)]
pub struct P4CmdHandler {
    /// Any warning handler that returns error will break the execution.
    pub warning_handlers: Vec<Box<dyn Handler>>,
    /// info about certain acrions not being done to individual files.
    pub info_handlers: Vec<Box<dyn Handler>>,
    /// The handler that can for example serialize the data or update some fields etc.
    pub output_handlers: Vec<Box<dyn Handler>>,
}

impl Handler for P4CmdHandler {
    fn process(&mut self, value: &String) -> Result<(), String> {
        if let Ok(warning_message) = serde_json::from_str::<WarningMessage>(value.as_str()) {
            // potentially expensive use this to detect if something is a warning message
            println!("{:?}", warning_message);
            return Ok(());
        }

        if let Ok(info_message) = serde_json::from_str::<InfoMessage>(value.as_str()) {
            println!("{:?}", info_message);
            return Ok(());
        }

        // TODO: call this only if there is no output handlers registered for the command.
        println!("unhandled output:\n{}", value);

        Ok(())
    }
}
