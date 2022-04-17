use std::process::Command;

use which::which;

use crate::error::{DurandalError, Result};

pub struct ExternalCommand {
    executable: String,
    args: Vec<String>,
}

impl ExternalCommand {
    pub fn new() -> ExternalCommandBuilder {
        ExternalCommandBuilder::default()
    }

    /// Run the external command and exit with its status code
    ///
    /// This function invokes `std::process::exit` and, as such, will terminate
    /// the current process.
    pub fn run(&self) -> ! {
        let result = Command::new(&self.executable)
            .args(&self.args)
            .status()
            .expect("There was an error in running the external command");
        std::process::exit(result.code().unwrap())
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ExternalCommandBuilder {
    prefix: String,
    name: String,
    args: Vec<String>,
}

impl ExternalCommandBuilder {
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn args(mut self, args: &[String]) -> Self {
        self.args = args.into();
        self
    }

    pub fn build(&self) -> Result<ExternalCommand> {
        let executable = format!("{}-{}", self.prefix, self.name);

        if let Err(_) = which(executable.clone()) {
            return Err(DurandalError::UnknownExternalCommand(executable));
        }

        Ok(ExternalCommand {
            executable,
            args: self.args.clone(),
        })
    }
}
