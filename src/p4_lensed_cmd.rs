//! Defines struct that helps defined Lensed TypeState Commands for strict control on what can be ran on them.
//! This approach still allows anything implementing P4Cmd and P4FormCmd to run all the methods, these only allow filtering when returned by factory methods.

use std::marker::PhantomData;

use crate::{
    error::P4Error,
    p4_cmd::{P4Cmd, P4FormCmd},
    p4_running_cmd::P4TypedRunningCmd,
};

// Marker traits to make the lensing more strict, without this anything would be allowed.

/// Used to limit / group variables on the command to only the compatible ones based on cmd synoptics.
pub trait P4CmdMode {}

/// Used to limit which final execution invocations will be available for the user.
pub trait P4RunMode {}

// Marker structs for valid command run modes.
pub struct P4Run;
impl P4RunMode for P4Run {}

pub struct P4RunSubmit;
impl P4RunMode for P4RunSubmit {}

pub struct P4RunRead;
impl P4RunMode for P4RunRead {}

/// No terminal methods. Used by `wild_west()` builders where the user must
/// call `.into_cmd()` to obtain the inner command and execute via trait methods.
pub struct P4RunNone;
impl P4RunMode for P4RunNone {}

///////////////////////////////////////////////////////////////////////////////////

/// Lenses are helping to limit what is visible for each command.
/// They provide a three axis composable restriction without creating new structs and allowing surfacing.
/// Goal is to provide the ability to limit which functions are available for user.
///
/// The execution methods here mirror those on `P4Cmd` and `P4FormCmd` traits.
/// If those trait signatures change, the corresponding methods here must be updated to match.
pub struct P4LensedCmd<C, M: P4CmdMode, R: P4RunMode> {
    pub(crate) inner_cmd: C,
    _phantom_data: PhantomData<(M, R)>,
}

impl<C, M: P4CmdMode, R: P4RunMode> P4LensedCmd<C, M, R> {
    pub fn new(cmd: C) -> Self {
        Self {
            inner_cmd: cmd,
            _phantom_data: PhantomData,
        }
    }

    pub fn into_cmd(self) -> C {
        self.inner_cmd
    }
}

/// Making it faster to create P4CmdModes by providing an easy way to provide a passthrough
///
/// Example for exposing a function of the inner_cmd:
/// ```rust,ignore
/// expose!(with_user, u: String);
/// ```
///
macro_rules! p4_expose {
    ($method:ident) => {
        pub fn $method(mut self) -> Self {
            self.inner_cmd = self.inner_cmd.$method();
            self
        }
    };
    ($method:ident, $($arg: ident: $ty:ty), +) => {
        pub fn $method(mut self, $($arg: $ty), +) -> Self {
            self.inner_cmd = self.inner_cmd.$method($($arg),+);
            self
        }
    }
}

/// Declares a single lensed command mode: the marker struct, its P4CmdMode impl, and the type alias.
/// Place directly above the corresponding `impl` block for that alias.
macro_rules! p4_lens {
    ($alias:ident, $cmd:ty, $mode:ident, $run:ty) => {
        pub struct $mode;
        impl P4CmdMode for $mode {}
        type $alias = P4LensedCmd<$cmd, $mode, $run>;
    };
}

///////////////////////////////////////////////////////////////////////////////////

// P4Cmd passthrough; mirrors P4Cmd trait methods.
// If P4Cmd changes, update these impls.

impl<C: P4Cmd, M: P4CmdMode> P4LensedCmd<C, M, P4Run> {
    pub fn run(self) -> Result<P4TypedRunningCmd<C::Output>, P4Error> {
        self.inner_cmd.run()
    }
}

// P4FormCmd passthrough; mirrors P4FormCmd trait methods.
// If P4FormCmd changes, update these impls.

impl<C: P4FormCmd, M: P4CmdMode> P4LensedCmd<C, M, P4RunRead> {
    pub fn read(self) -> Result<C::Form, P4Error> {
        self.inner_cmd.read_form()
    }
}

impl<C: P4FormCmd, M: P4CmdMode> P4LensedCmd<C, M, P4RunSubmit> {
    pub fn submit(self, form: C::Form) -> Result<C::FormSubmitOutput, P4Error> {
        self.inner_cmd.submit_form(form)
    }
}
