use anyhow::Result;
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Select};
use durandal_core::CliMetaCommand;

use crate::{config::Config, task, workflow::force_next_task};

/// Replan a project by maybe changing the task that is next.
#[derive(Args)]
pub struct Replan;

impl CliMetaCommand for Replan {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let projects = task::projects_excluding(&config.excluded_projects)?;

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which project to replan?")
            .items(&projects)
            .default(0)
            .interact()?;

        if let Some(project) = projects.get(choice) {
            // if the selected project already has a next task, we need to
            // remove the next tag

            force_next_task(project.clone())?;
        }

        Ok(())
    }
}
