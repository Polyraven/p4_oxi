//! Defines core traits and a general wrapper around P4 commands that the whole command system in this crate is built on..

use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    error::P4Error,
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
/// P4RawCmd::new("info").single_content()?
///```
///
/// Different part of the command are set by individual functions.
/// ```ignore
///     P4RawCmd::new("fstat")
///     .wildcard(r"c:\my_workspace\my_directory\...")
///     .with_arg_value("-m", "5")
///     .start()?.for_each(|_| {});
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
    pub form: Option<Box<dyn P4Form>>,
    ///
    pub form_action: P4FormAction,
}

#[derive(Default)]
pub enum P4FormAction {
    #[default]
    None,
    Submit,
    Read,
}

/// When dealing with forms they need to be uploaded or downloaded with specific flags.
/// This ensures those are set properly.
impl P4FormAction {
    fn to_arg_list(&self) -> Vec<String> {
        match self {
            P4FormAction::None => vec![],
            P4FormAction::Read => vec!["-o".to_string()],
            P4FormAction::Submit => vec!["-i".to_string()],
        }
    }
}

impl P4RawCmd {
    pub fn new(cmd: impl ToString) -> Self {
        P4RawCmd {
            cmd: cmd.to_string(),
            ..Default::default()
        }
    }

    pub fn with_g_options(mut self, g_options: GlobalOptions) -> Self {
        self.g_option = g_options;
        self
    }

    pub fn with_arg(mut self, arg: impl ToString) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn with_arg_value(mut self, arg: impl ToString, value: impl ToString) -> Self {
        self.args.push(arg.to_string());
        self.args.push(value.to_string());
        self
    }

    pub fn with_args(mut self, args: &[impl ToString]) -> Self {
        args.iter().for_each(|arg| self.args.push(arg.to_string()));
        self
    }

    pub fn with_wildcard(mut self, wildcard: impl Into<String>) -> Self {
        self.wildcards.push(wildcard.into());
        self
    }

    pub fn with_wildcards(mut self, wildcards: &[impl AsRef<str>]) -> Self {
        wildcards
            .iter()
            .for_each(|wildcard| self.wildcards.push(wildcard.as_ref().to_string()));
        self
    }

    pub fn with_submit_form(mut self, form: Box<dyn P4Form>) -> Self {
        self.form_action = P4FormAction::Submit;
        self.form = Some(form);
        self.g_option.format_as_json = false;
        self
    }

    pub fn with_read_form(mut self) -> Self {
        self.form_action = P4FormAction::Read;
        self.form = None;
        self.g_option.format_as_json = true;
        self
    }

    /// This is the default state, no need to call it unless mutating an existing command.
    pub fn with_no_form_action(mut self) -> Self {
        self.form_action = P4FormAction::None;
        self.form = None;
        self.g_option.format_as_json = true;
        self
    }

    /// Will start a P4RunningCmd that can be used as iterator. Or .collected() immediately.
    /// At this point the output one iterates on is not typed, so each item would be the pure output per item as P4 returns it.
    /// Howver one can use .cast() to get a concrete type. This is what P4Cmd and P4FormCmd traits already do with the output type specified.
    pub fn run(self) -> Result<P4RunningCmd, P4Error> {
        if self.form.is_some() || matches!(self.form_action, P4FormAction::Submit) {
            return Err(P4Error::InvalidUsage(
                "A command cannot be run() when input_form is set (unintended misuse? For uploading forms call .submit_form())",
            ));
        }

        let iter_cmd = IterCmd::from(self).run()?;
        Ok(P4RunningCmd {
            inner_iter_cmd: iter_cmd,
        })
    }

    /// Submits the form to the server and returns the raw output.
    /// ```ignore
    /// let form = P4ChangeForm::new("my_changelist")?;
    /// let output = P4RawCmd::new("change")
    ///    .with_submit_form(Box::new(form))
    ///    .submit_form()?;
    /// ```
    /// Note that forms should implement P4FormCmd and P4RawCmd would not need to be used.
    pub fn submit_form(self) -> Result<String, P4Error> {
        if self.form.is_none() || !matches!(self.form_action, P4FormAction::Submit) {
            return Err(P4Error::InvalidUsage(
                ".submit_form() was called while the form and/or form_action is unset. Use .with_submit_form()",
            ));
        }

        IterCmd::from(self).send_input()
    }
}

impl From<P4RawCmd> for IterCmd {
    fn from(p4_cmd: P4RawCmd) -> Self {
        let mut cmd = IterCmd::new("p4")
            .args(&p4_cmd.g_option.to_arg_list())
            .arg(&p4_cmd.cmd)
            .args(&p4_cmd.form_action.to_arg_list())
            .args(&p4_cmd.args)
            .args(&p4_cmd.wildcards);

        if let Some(input_form) = p4_cmd.form {
            cmd = cmd.set_input(input_form.convert());
        }
        cmd
    }
}

/// Trait for commands that deserialize the server output to the specified output type.
///
/// To make these methods available on a command one would add this during implementation:
/// ```rust, no_run
/// impl P4Cmd for P4FStat {
///    type Output = P4FStatOut;
/// ```
///
/// Note: If method signatures change here, update the corresponding passthrough impls in `P4LensedCmd`.
pub trait P4Cmd: Into<P4RawCmd> {
    type Output: DeserializeOwned;

    fn run(self) -> Result<P4TypedRunningCmd<Self::Output>, P4Error> {
        Ok(self.into().run()?.cast())
    }

    /// Returns the first item, expecting it to be Content.
    /// Fails if the first item is a non-content message or if there are no results.
    fn single_content(self) -> Result<Self::Output, P4Error> {
        match self.run()?.next() {
            Some(P4Out::Content(v)) => Ok(v),
            Some(other) => Err(P4Error::UnexpectedOutput(other.erase_content_type())),
            None => Err(P4Error::NoResults),
        }
    }

    /// Skips Info and Warning messages, returning the first Content item.
    /// Fails on Failure/ParseError, or if no Content is found.
    fn first_content(self) -> Result<Self::Output, P4Error> {
        for item in self.run()? {
            match item {
                P4Out::Content(v) => return Ok(v),
                P4Out::Info(_) | P4Out::Warning(_) => continue,
                other => return Err(P4Error::UnexpectedOutput(other.erase_content_type())),
            }
        }
        Err(P4Error::NoResults)
    }

    /// Executes the command without returning anything, usefull for fire and forget commands.
    /// If the return value is required, use run()
    fn apply(self) -> Result<(), P4Error> {
        for item in self.run()? {
            match item {
                P4Out::Failure(_) | P4Out::ParseError(_) => {
                    return Err(P4Error::UnexpectedOutput(item.erase_content_type()));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Trait for commands that read or submit P4 forms.
///
/// Example implementation for a command:
/// impl P4FormCmd for P4Change {
///     type FormSubmitOutput = u64;
///     type Form = P4ChangeForm;
///
///     fn parse_form_submit_raw_output(raw: String) -> Result<Self::FormSubmitOutput, P4Error> {
///          // Parse into FormSubmitOutput here
///     }
/// }
///
/// Note: If method signatures change here, update the corresponding passthrough impls in `P4LensedCmd`.
pub trait P4FormCmd: Into<P4RawCmd> {
    type FormSubmitOutput;
    type Form: P4Form + DeserializeOwned + Debug + 'static;

    fn read_form(self) -> Result<Self::Form, P4Error> {
        let raw_command = self.into().with_read_form();
        match raw_command.run()?.cast::<Self::Form>().next() {
            Some(P4Out::Content(form)) => Ok(form),
            Some(other) => Err(P4Error::UnexpectedOutput(other.erase_content_type())),
            None => Err(P4Error::NoResults),
        }
    }

    fn submit_form(self, form: Self::Form) -> Result<Self::FormSubmitOutput, P4Error> {
        let raw_output = self.into().with_submit_form(Box::new(form)).submit_form()?;
        Self::parse_form_submit_raw_output(raw_output)
    }

    fn parse_form_submit_raw_output(raw_output: String) -> Result<Self::FormSubmitOutput, P4Error>;
}

/// A trait for structures that hold data of a P4 Forms and need to be converted to the CLI commands.
pub trait P4Form {
    fn convert(&self) -> String;
}
