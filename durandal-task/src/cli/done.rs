use anyhow::{bail, Result};
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, Processable, Project},
    workflow::set_next_task,
};

/// Mark the curently active task as done, selecting a new next task if possible.
///
/// If the completed task's project has other pending tasks, will prompt for
/// which of those should be the next task for that project.
#[derive(Args)]
pub struct Done;

impl CliMetaCommand for Done {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let candidates = active_tasks()?;

        if candidates.is_empty() {
            bail!("No active task");
        }

        if candidates.len() > 1 {
            bail!("More than one active task detected. Aborting");
        }

        // this unwrap is "safe" because of the previous two checks
        let active = candidates.first().unwrap();

        println!(
            "{}",
            style(format!("Finishing: {}", active.description())).green()
        );
        active.finish()?;

        if let Some(resume) = active.task_to_resume()? {
            // If we had a task to resume, do so then exit
            println!(
                "{}",
                style(format!("Resuming previous task: {}", resume.description())).magenta()
            );
            resume.begin()?;
            return Ok(());
        }

        // at this point in time the id is invalid, but the other fields are
        // mostly still fine
        if let Some(proj) = active.project().cloned() {
            let project = Project::from(proj);
            if !project.tasks()?.is_empty() {
                set_next_task(project)?;
            }
        }

        Ok(())
    }
}
