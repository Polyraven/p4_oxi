//! Defines the various command runners which is using the typestate decorator pattern.
//!
//! Here us what's happening
//! When .run() is called on a P4RawCmd it first creates an IterCmd
//!
//! The full chain:
//!     RunningIterCmd -> P4RunningCmd -> P4TypedRunningCmd
//!
//! How it works:
//! The RunningIterCmd outputs raw data without any processing.
//! The P4RunningCmd splits the content from warnings (which are returned as P4Message) # TODO rename it to P4Error
//! The P4TypedRunningCmd can be created from this using the .cast() method
//! That in the end will make each content to be deserialized to a concrete type.

use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use crate::iter_cmd::RunningIterCmd;
use crate::p4_out::{Failure, InfoMessage, P4Out, ParseError, WarningMessage};

/// An iterator over a running command that categorizes the output messages between content and warnings.
///
/// It emits Result<String, P4Message>
/// Where:
///     String: content in raw format
///     P4Message: warnings and various error messages
pub struct P4RunningCmd {
    pub inner_iter_cmd: RunningIterCmd,
}

impl P4RunningCmd {
    /// Convert to P4TypedRunningCmd that will be able to deserialize each output to the specified type.
    pub fn cast<T>(self) -> P4TypedRunningCmd<T> {
        P4TypedRunningCmd {
            inner_p4_running_cmd: self,
            _phantom: PhantomData,
        }
    }
}

impl Iterator for P4RunningCmd {
    type Item = P4Out<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(value) = self.inner_iter_cmd.next() else {
                return None;
            };

            match value {
                Err(e) => return Some(P4Out::Failure(Failure { data: e })),
                Ok(line) => {
                    if let Ok(warning_message) = serde_json::from_str::<WarningMessage>(&line) {
                        return Some(P4Out::Warning(warning_message));
                    }

                    if let Ok(info_message) = serde_json::from_str::<InfoMessage>(&line) {
                        return Some(P4Out::Info(info_message));
                    }

                    return Some(P4Out::Content(line));
                }
            }
        }
    }
}

/// A type bound iterator over a running command that deserializes each output to the type it is bound to.
///
/// It emits Result<T, P4Message>
/// Where:
///     - T is the successfully deserialized content
///     - P4Message warnings and various error messages
///
pub struct P4TypedRunningCmd<T> {
    inner_p4_running_cmd: P4RunningCmd,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> Iterator for P4TypedRunningCmd<T> {
    type Item = P4Out<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner_p4_running_cmd.next()? {
            P4Out::Content(line) => match serde_json::from_str::<T>(&line) {
                Ok(val) => Some(P4Out::Content(val)),
                Err(e) => Some(P4Out::ParseError(ParseError {
                    data: line,
                    error: e.to_string(),
                })),
            },
            P4Out::Warning(w) => Some(P4Out::Warning(w)),
            P4Out::Info(i) => Some(P4Out::Info(i)),
            P4Out::Failure(f) => Some(P4Out::Failure(f)),
            P4Out::ParseError(e) => Some(P4Out::ParseError(e)),
        }
    }
}
