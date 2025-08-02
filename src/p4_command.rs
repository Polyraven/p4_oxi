use std::any::Any;

use crate::{
    global_options::GlobalOptions,
    handled_command::{HandledCommand, Handler},
};

pub trait P4Command {
    fn prefix(&self) -> HandledCommand {
        HandledCommand::new("p4").args(&GlobalOptions::default().to_arg_list())
    }

    fn build(self) -> HandledCommand;

    fn execute(self) -> Result<Vec<String>, String>;
}
