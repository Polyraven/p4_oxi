use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;

use crate::{
    handled_command::{HandledCmdOut, Handler, ProcessingResult},
    warnings::{InfoMessage, WarningMessage},
};

pub trait WarningHandler {
    fn process(&mut self, value: &WarningMessage) -> Result<(), String>;
}

pub trait InfoHandler {
    fn process(&mut self, value: &InfoMessage) -> Result<(), String>;
}

pub trait OutHandler {
    fn process(&mut self, value: &String) -> Result<(), String>;
}

pub trait P4OutputParsing<T> {
    fn parse(self) -> Result<Vec<T>, String>;
}

pub trait P4Single<T> {
    fn single(self) -> Result<T, String>;
}

#[derive(Default)]
pub struct P4CmdHandlers {
    /// Any warning handler that returns error will break the execution.
    pub warning_handlers: Vec<Box<dyn WarningHandler>>,
    /// info about certain acrions not being done to individual files.
    pub info_handlers: Vec<Box<dyn InfoHandler>>,
    /// The handler that can for example serialize the data or update some fields etc.
    pub output_handlers: Vec<Box<dyn OutHandler>>,
}

impl Handler for P4CmdHandlers {
    fn process(&mut self, value: &String) -> Result<ProcessingResult, String> {
        // potentially expensive use this to detect if something is a warning message
        if let Ok(warning_message) = serde_json::from_str::<WarningMessage>(value.as_str()) {
            // TODO: WarningHandler here: Report / Silent / Error (stop) + Allow multiple
            return Ok(ProcessingResult::Warning(warning_message.data));
        }

        if let Ok(info_message) = serde_json::from_str::<InfoMessage>(value.as_str()) {
            println!("{:?}", info_message);
            // TODO: InfoHandler here: Report / Silent / Error (stop) + Allow multiple
            return Ok(ProcessingResult::Silent);
        }

        for output_handler in &mut self.output_handlers {
            // TODO: OutHandler here: Report / Silent / Error (stop) + Allow multiple
            if let Err(e) = output_handler.process(value) {
                return Err(format!("Error while processing output: {}", e));
            }
        }

        Ok(ProcessingResult::Content(value.clone()))
    }
}

// Will deserialize each message received back by the command into any type that implements Deserialize
// The fields of those types are expected to me matched precisely to the response of P4V they meant to process
// Otherwise it will be returning an Error when such a message is encountered
pub struct DeserializingHandler<T: DeserializeOwned> {
    // the result is safe to be acessed from a thread if need be
    pub results: Arc<Mutex<Vec<T>>>,
}

impl<T: DeserializeOwned> DeserializingHandler<T> {
    pub fn new() -> Box<Self> {
        let result = Arc::new(Mutex::new(Vec::new()));

        let handler = Self {
            results: Arc::clone(&result),
        };

        return Box::new(handler);
    }

    pub fn result_arc_clone(&self) -> Arc<Mutex<Vec<T>>> {
        Arc::clone(&self.results)
    }
}

impl<T: DeserializeOwned> OutHandler for DeserializingHandler<T> {
    fn process(&mut self, value: &String) -> Result<(), String> {
        match serde_json::from_str::<T>(value.as_str()) {
            Ok(parsed) => {
                self.results.lock().unwrap().push(parsed);
                return Ok(());
            }
            Err(e) => {
                return Err(format!(
                    "Failed to deserialized output value.\nerror: {}\nvalue: {}",
                    e, value
                ));
            }
        }
    }
}

impl<T: DeserializeOwned> P4OutputParsing<T> for HandledCmdOut {
    fn parse(self) -> Result<Vec<T>, String> {
        let mut result: Vec<T> = Vec::new();
        for content_value in self.content {
            match serde_json::from_str::<T>(content_value.as_str()) {
                Err(e) => {
                    return Err(format!(
                        "Failed to serialize \n error: {} \n value: {} |",
                        e, content_value
                    ));
                }
                Ok(serialized) => result.push(serialized),
            }
        }
        Ok(result)
    }
}

impl<T: DeserializeOwned> P4Single<T> for Vec<T> {
    fn single(self) -> Result<T, String> {
        let count = self.iter().len();
        if count != 1 {
            return Err(format!("The number of results is not one ({})", count));
        };
        self.into_iter()
            .next()
            .ok_or_else(|| "Failed to get info from P4".to_string())
    }
}
