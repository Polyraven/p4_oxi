//! Defines core traits and a general wrapper around P4 commands that the whole command system in this crate is built on..

use serde::de::DeserializeOwned;

use crate::{
    global_options::GlobalOptions,
    iter_cmd::IterCmd,
    p4_out::P4Out,
    p4_running_cmd::{P4RunningCmd, P4TypedRunningCmd},
};

/// A General low-level wrapper around P4 CLI commands.
///
/// Other high level commands in the end convert to this.
/// This here meant to be able to help run any command which is not or only partially implemented in .cmd
///
/// This struct is using the builder pattern where one would construct a command and than execute it.
/// Example:
/// ```ignore
/// P4RawCmd::new("info").run()?.single()
///```
///
/// Different part of the command are set by individual functions.
/// ```ignore
///     P4RawCmd::new("fstat")
///     .wildcard(r"c:\my_workspace\my_directory\...")
///     .with_arg_value("-m", "5")
///     .start()?.for_each(|fstat| {});
///```
///
/// The `.start()?` here starts the executions and provides an iterator while p4 is still running.
/// Thus providing effortless concurency without the need of threads.
///
///
/// CmdRef documentation:
/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html
#[derive(Default)]
pub struct P4RawCmd {
    pub g_option: GlobalOptions,
    /// The P4 command that will be executed, like fstat, sync, clean etc.
    pub cmd: String,
    /// For the arguments that start with a `-`
    pub args: Vec<String>,
    /// Last argument, needs special validation
    pub wildcards: Vec<String>,
    /// Used when creating a changelist or a stream etc. by feeding the bytes in with stdin.
    pub input_sepcification: Option<Box<dyn P4Specification>>,
}

impl P4RawCmd {
    pub fn new(cmd: &str) -> Self {
        P4RawCmd {
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

    pub fn with_input_spec(mut self, input_spec: Box<dyn P4Specification>) -> Self {
        self.g_option.format_as_json = false;
        self.input_sepcification = Some(input_spec);
        self
    }

    /// Will start a P4RunningCmd that can be used as iterator. Or .collected() immediately.
    /// At this point the output one iterates on is not typed, so each item would be the pure output per item as P4 returns it.
    /// Howver one can use .cast() to get a concrete type. This is what P4Cmd and P4SpecCmd traits already do with the output type specified.
    /// Will start a P4RunningCmd that can be used as iterator. Or .collected() immediately.
    /// At this point the output one iterates on is not typed, so each item would be the pure output per item as P4 returns it.
    /// Howver one can use .cast() to get a concrete type. This is what P4Cmd and P4SpecCmd traits already do with the output type specified.
    pub fn run(self) -> Result<P4RunningCmd, String> {
        if self.input_sepcification.is_some() {
            return Err(
                "A command cannot be called when input_spec is set (unintended missue? For uploading specs call .submit_spec())".to_string(),
            );
        }

        let iter_cmd = IterCmd::from(self).run()?;
        Ok(P4RunningCmd {
            inner_iter_cmd: iter_cmd,
        })
    }

    pub fn submit_spec(self) -> Result<String, String> {
        if self.input_sepcification.is_none() {
            return Err(".submit_spec() was called while the specification is empty.".to_string());
        }

        // returns the raw data for now, but the Specification should implement what this returns.
        // Also note thet send_input will use change now right away.
        IterCmd::from(self).send_input()
    }
}

impl From<P4RawCmd> for IterCmd {
    fn from(p4_cmd: P4RawCmd) -> Self {
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

pub trait P4Cmd {
    type Output;

    fn to_p4_query_cmd(&self) -> P4RawCmd;

    fn run(&self) -> Result<P4TypedRunningCmd<Self::Output>, String>
    where
        Self::Output: DeserializeOwned,
    {
        Ok(self.to_p4_query_cmd().run()?.cast())
    }

    fn single(&self) -> Result<Self::Output, String>
    where
        Self::Output: DeserializeOwned,
    {
        self.run()?
            .next()
            .ok_or_else(|| "Expected a single result but got none".to_string())?
            .map_err(|e| format!("{:?}", e))
    }
}

pub trait P4SpecCmd {
    type Output;

    fn to_p4_spec_submit_cmd(&self) -> P4RawCmd;

    fn submit_spec(&self, spec: Box<dyn P4Specification>) -> Result<String, String> {
        self.to_p4_spec_submit_cmd()
            .with_input_spec(spec)
            .submit_spec()
    }
}

/// A trait for structures that hold P4 Specs and need to be converted to the CLI commands.
pub trait P4Specification {
    fn convert(&self) -> String;
}
