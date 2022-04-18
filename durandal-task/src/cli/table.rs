use anyhow::{bail, Result};
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, Processable},
};

/// "Table" the active task by stopping work and removing the next tag.
#[derive(Args)]
pub struct Table;

impl CliMetaCommand for Table {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let candidates = active_tasks()?;

        if candidates.is_empty() {
            bail!("No active task");
        }

        if candidates.len() > 1 {
            bail!("More than one active task detected. Aborting");
        }

        let active = candidates.first().unwrap();

        println!(
            "{}",
            style(format!("Tabling: {}", active.description()))
                .dim()
                .yellow()
        );

        active.stop()?;
        active.remove_tags(&["next"])?;

        Ok(())
    }
}
