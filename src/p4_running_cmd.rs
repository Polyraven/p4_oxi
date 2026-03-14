use serde::de::DeserializeOwned;

use crate::iter_cmd::RunningIterCmd;
use crate::warnings::{Failure, InfoMessage, P4Message, ParseError, WarningMessage};

pub struct P4RunningCmd<T> {
    inner: RunningIterCmd,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> P4RunningCmd<T> {
    pub fn new(inner: RunningIterCmd) -> Self {
        Self {
            inner,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next_raw(&mut self) -> Option<Result<String, P4Message>> {
        loop {
            let Some(value) = self.inner.next() else {
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

impl<T: DeserializeOwned> Iterator for P4RunningCmd<T> {
    type Item = Result<T, P4Message>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_raw()? {
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
