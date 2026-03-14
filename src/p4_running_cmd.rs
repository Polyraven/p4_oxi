use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use crate::iter_cmd::RunningIterCmd;
use crate::warnings::{Failure, InfoMessage, P4Message, ParseError, WarningMessage};

pub struct P4RunningCmd {
    pub inner_iter_cmd: RunningIterCmd,
}

impl P4RunningCmd {
    pub fn cast<T>(self) -> P4TypedRunningCmd<T> {
        P4TypedRunningCmd {
            inner_p4_running_cmd: self,
            _phantom: PhantomData,
        }
    }
}

impl Iterator for P4RunningCmd {
    type Item = Result<String, P4Message>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(value) = self.inner_iter_cmd.next() else {
                return None;
            };

            match value {
                Err(e) => return Some(Err(P4Message::Failure(Failure { data: e }))),
                Ok(line) => {
                    if let Ok(warning_message) = serde_json::from_str::<WarningMessage>(&line) {
                        return Some(Err(P4Message::Warning(warning_message)));
                    }

                    if let Ok(info_message) = serde_json::from_str::<InfoMessage>(&line) {
                        return Some(Err(P4Message::Info(info_message)));
                    }

                    return Some(Ok(line));
                }
            }
        }
    }
}

pub struct P4TypedRunningCmd<T> {
    inner_p4_running_cmd: P4RunningCmd,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> Iterator for P4TypedRunningCmd<T> {
    type Item = Result<T, P4Message>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner_p4_running_cmd.next()? {
            Err(msg) => Some(Err(msg)),
            Ok(line) => Some(serde_json::from_str::<T>(&line).map_err(|e| {
                P4Message::ParseError(ParseError {
                    data: line,
                    error: e.to_string(),
                })
            })),
        }
    }
}
