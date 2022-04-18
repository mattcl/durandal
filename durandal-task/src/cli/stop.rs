use anyhow::{bail, Result};
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, Processable},
};

/// Stops the ACTIVE task(s), if any.
#[derive(Args)]
pub struct Stop;

impl CliMetaCommand for Stop {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let active = active_tasks()?;

        if active.is_empty() {
            bail!("No active task");
        }

        for task in &active {
            task.stop()?;
            println!(
                "{}",
                style(format!(
                    "Stopped {} {}",
                    task.id().unwrap_or_default(),
                    task.description()
                ))
                .green()
            );
        }

        Ok(())
    }
}
