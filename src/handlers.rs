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

impl<'a> Handler for P4CmdHandler {
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

pub struct OutputHandler<T: for<'de> serde::de::Deserialize<'de>> {
    pub results: Vec<T>,
}

impl<T: for<'de> serde::de::Deserialize<'de>> OutputHandler<T> {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn single(&mut self) -> Result<&T, String> {
        if self.results.len() != 1 {
            return Err("Non single result found.".to_string());
        };

        return self
            .results
            .iter()
            .next()
            .ok_or("Single result failed to return.".to_string());
    }
}

impl<T: for<'de> serde::de::Deserialize<'de>> Handler for OutputHandler<T> {
    fn process(&mut self, value: &String) -> Result<(), String> {
        match serde_json::from_str::<T>(value.as_str()) {
            Ok(parsed) => {
                self.results.push(parsed);
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
