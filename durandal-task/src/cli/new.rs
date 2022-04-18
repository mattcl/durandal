use crate::{config::Config, workflow::new_task};
use anyhow::Result;
use clap::Args;
use durandal_core::CliMetaCommand;

/// Convenience for adding a new deferred task.
#[derive(Args)]
pub struct New;

impl CliMetaCommand for New {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        new_task()?;
        Ok(())
    }
}
