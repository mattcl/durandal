//! This module provides functionality related to running external subcommands
use std::process::Command;

use which::which;

use crate::error::{DurandalError, Result};

/// This struct supports the execution of an external command.
///
/// It is constructed via the [ExternalCommandBuilder]. Assuming that
/// successfully yields and instance, the external command's
/// [run](ExternalCommand::run) method will execute the external command,
/// exiting the current process when that command terminates. The current
/// process will be exited with the external command's exit code.
///
/// The builder will yield an error in the even the external command cannot be
/// located.
///
/// # Examples
/// ```
/// use durandal_core::external::ExternalCommand;
/// let cmd = ExternalCommand::new()
///     .prefix("hopeit")
///     .name("ismissing")
///     .build();
///
/// // we expect the failure, since hopeit-ismissing will hopefully not be in
/// // the PATH
/// assert!(cmd.is_err());
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
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

/// This builder yields instances of [ExternalCommand].
///
/// The builder will yield an error in the event the computed external command
/// cannot be found in the user's executable path. See [ExternalCommand] for
/// an example of the builder usage.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ExternalCommandBuilder {
    prefix: String,
    name: String,
    args: Vec<String>,
}

impl ExternalCommandBuilder {
    /// Set the prefix for the external command
    ///
    /// We expect external commands to be in the form `PREFIX-NAME`
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Set the name for the external command
    ///
    /// We expect external commands to be in the form `PREFIX-NAME`
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set the args for the external command
    pub fn args(mut self, args: &[String]) -> Self {
        self.args = args.into();
        self
    }

    /// Attempt to construct an [ExternalCommand] from the builder.
    ///
    /// This will return an error in the even the external command cannot be
    /// found in the user's executable path. Namely, this returns
    /// [DurandalError::UnknownExternalCommand] in that case.
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
