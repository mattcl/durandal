use anyhow::Result;
use clap::Args;
use comfy_table::Color;
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::active_tasks,
    task_table::{display_table, Field},
};

/// Display the ACTIVE task, if one exists.
#[derive(Args)]
pub struct Current;

impl CliMetaCommand for Current {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let active = active_tasks()?;
        let cols = vec![Field::ID, Field::Project, Field::AnnotatedDescription];
        display_table(&active, &cols, Color::DarkYellow);

        Ok(())
    }
}
