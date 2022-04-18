use anyhow::Result;
use clap::Args;
use comfy_table::Color;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, load_tasks, Processable},
    task_table::{display_table, Field},
};

/// Find something to work on.
#[derive(Args)]
pub struct Next;

impl CliMetaCommand for Next {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let active = active_tasks()?;
        if !active.is_empty() {
            let cols = vec![Field::ID, Field::Project, Field::AnnotatedDescription];
            // we already have tasks in progress
            println!(
                "{}",
                style("Cannot start a task when other task(s) are active:").red()
            );
            display_table(&active, &cols, Color::Magenta);

            // TODO: this probably shouldn't be 0 - MCL - 2022-03-20
            return Ok(());
        }

        let next_task_filter = "+next -ACTIVE status:pending";
        let next_tasks = load_tasks(next_task_filter)?;
        let mut choices: Vec<_> = next_tasks
            .iter()
            .map(|t| {
                format!(
                    "{} {}: {}",
                    t.id().unwrap_or_default(),
                    t.project().cloned().unwrap_or_default(),
                    t.description()
                )
            })
            .collect();

        choices.push("-- nothing --".into());

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to work on?")
            .items(&choices)
            .default(0)
            .interact()?;

        // if we don't match anything, we don't care, because it's the nothing
        // option
        if let Some(task) = next_tasks.get(choice) {
            task.begin()?;
            println!(
                "{}",
                style(format!("started {}", task.id().unwrap_or_default())).green()
            );
        }

        Ok(())
    }
}
