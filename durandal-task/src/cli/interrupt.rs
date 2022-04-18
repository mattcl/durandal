use anyhow::Result;
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, Processable},
    workflow::add_to_project,
};

/// Creates and starts an interrupt task.
///
/// If there is an ACTIVE task, this command will stop that task prior to
/// starting the new task.
#[derive(Args)]
pub struct Interrupt;

impl CliMetaCommand for Interrupt {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        // 1. stop any started tasks
        // 2. create a new task
        // 3. start the new task
        let active = active_tasks()?;
        let mut resume: Option<_> = None;
        if !active.is_empty() {
            for task in &active {
                // we can only resume the first active task
                if resume.is_none() {
                    resume = Some(task);
                }

                task.stop()?;
                println!(
                    "{}",
                    style(format!(
                        "Stopped {} {}",
                        task.id().unwrap_or_default(),
                        task.description()
                    ))
                    .yellow()
                );
            }
        }

        // lets just have the project and tags signal interrupt
        let task = add_to_project("interrupt".into())?;
        task.add_tags(&["interrupt"])?;

        if let Some(resume) = resume {
            task.set_resumable(resume)?;
        }

        task.begin()?;

        println!(
            "{}",
            style(format!("started {}", task.id().unwrap_or_default())).green()
        );

        Ok(())
    }
}
