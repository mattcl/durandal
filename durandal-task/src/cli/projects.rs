use crate::{config::Config, task, workflow::set_next_task};
use anyhow::Result;
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;

/// Ensure projects have next actions.
#[derive(Args)]
pub struct Projects;

impl CliMetaCommand for Projects {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let projects = task::projects_excluding(&config.excluded_projects)?;

        for project in &projects {
            // ignore the empty project
            if project.len() < 1 {
                continue;
            }

            set_next_task(project.clone())?;
        }

        println!("{}", style("No remaining projects").yellow());

        Ok(())
    }
}
