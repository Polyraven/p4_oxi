//! Defines the various command runners which is using the typestate decorator pattern.
//!
//! When `.run()` is called on a `P4RawCmd` it creates a `RunningIterCmd`.
//!
//! The full chain:
//!     `RunningIterCmd` -> `P4RunningCmd` -> `P4TypedRunningCmd`
//!
//! How it works:
//! - `RunningIterCmd` outputs raw data without any processing.
//! - `P4RunningCmd` categorizes output into [`P4Out`] variants (content, warnings, info, failures).
//! - `P4TypedRunningCmd` deserializes each content item to a concrete type `T`.

use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use crate::error::P4Error;
use crate::iter_cmd::RunningIterCmd;
use crate::p4_out::{P4Failure, P4InfoMessage, P4Out, P4ParseError, P4WarningMessage};

/// An iterator over a running command that categorizes the output into [`P4Out`] variants.
///
/// Yields [`P4Out<String>`] where content is in raw (unparsed) format.
pub struct P4RunningCmd {
    pub(crate) inner_iter_cmd: RunningIterCmd,
}

impl P4RunningCmd {
    /// Convert to P4TypedRunningCmd that will be able to deserialize each output to the specified type.
    pub fn cast<T>(self) -> P4TypedRunningCmd<T> {
        P4TypedRunningCmd {
            inner_p4_running_cmd: self,
            _phantom: PhantomData,
        }
    }

    /// Returns the first item from the iterator, or [`P4Error::NoResults`] if empty.
    pub fn first(&mut self) -> Result<P4Out<String>, P4Error> {
        self.next().ok_or(P4Error::NoResults)
    }

    /// Returns the first and only item from the iterator.
    /// Errors if the iterator is empty or yields more than one item.
    pub fn single(&mut self) -> Result<P4Out<String>, P4Error> {
        let item = self.first()?;
        if let Some(extra) = self.next() {
            return Err(P4Error::UnexpectedOutput(extra.erase_content_type()));
        }
        Ok(item)
    }

    /// Consumes all remaining items, returning an error on Failure or ParseError.
    pub fn finish(&mut self) -> Result<(), P4Error> {
        for item in self {
            if matches!(item, P4Out::Failure(_) | P4Out::ParseError(_)) {
                return Err(P4Error::UnexpectedOutput(item.erase_content_type()));
            }
        }
        Ok(())
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
                Err(e) => return Some(P4Out::Failure(P4Failure { data: e })),
                Ok(line) => {
                    if let Ok(warning_message) = serde_json::from_str::<P4WarningMessage>(&line) {
                        return Some(P4Out::Warning(warning_message));
                    }

                    if let Ok(info_message) = serde_json::from_str::<P4InfoMessage>(&line) {
                        return Some(P4Out::Info(info_message));
                    }

                    return Some(P4Out::Content(line));
                }
            }
        }
    }
}

/// A typed iterator over a running command that deserializes each content item to `T`.
///
/// Yields [`P4Out<T>`] where content items are deserialized from JSON.
pub struct P4TypedRunningCmd<T> {
    inner_p4_running_cmd: P4RunningCmd,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> P4TypedRunningCmd<T> {
    /// Returns the first item from the iterator, or [`P4Error::NoResults`] if empty.
    pub fn first(&mut self) -> Result<P4Out<T>, P4Error> {
        self.next().ok_or(P4Error::NoResults)
    }

    /// Returns the first and only item from the iterator.
    /// Errors if the iterator is empty or yields more than one item.
    pub fn single(&mut self) -> Result<P4Out<T>, P4Error> {
        let item = self.first()?;
        if let Some(extra) = self.next() {
            return Err(P4Error::UnexpectedOutput(extra.erase_content_type()));
        }
        Ok(item)
    }

    /// Consumes all remaining items, returning an error on Failure or ParseError.
    pub fn finish(&mut self) -> Result<(), P4Error> {
        self.inner_p4_running_cmd.finish()
    }
}

impl<T: DeserializeOwned> Iterator for P4TypedRunningCmd<T> {
    type Item = P4Out<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner_p4_running_cmd.next()? {
            P4Out::Content(line) => match serde_json::from_str::<T>(&line) {
                Ok(val) => Some(P4Out::Content(val)),
                Err(e) => Some(P4Out::ParseError(P4ParseError {
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
