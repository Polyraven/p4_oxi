use std::sync::{Arc, Mutex};

use crate::{
    handled_command::Handler,
    warnings::{InfoMessage, WarningMessage},
};

#[derive(Default)]
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
        if self.output_handlers.is_empty() {
            println!("no output handlers:\n{}", value);
        }

        for output_handler in &mut self.output_handlers {
            // TODO collect these and continue? Or return early?
            if let Err(e) = output_handler.process(value) {
                return Err(format!("Error while processing output: {}", e));
            }
        }

        Ok(())
    }
}

// Will deserialize each message received back by the command into any type that implements Deserialize
// The fields of those types are expected to me matched precisely to the response of P4V they meant to process
// Otherwise it will be returning an Error when such a message is encountered
pub struct MsgHandler<T: for<'de> serde::de::Deserialize<'de>> {
    // the result is safe to be acessed from a thread if need be
    pub results: Arc<Mutex<Vec<T>>>,
}

impl<T: for<'de> serde::de::Deserialize<'de>> MsgHandler<T> {
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

impl<T: for<'de> serde::de::Deserialize<'de>> Handler for MsgHandler<T> {
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
