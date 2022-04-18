use crate::{
    config::Config,
    task::TaskClient,
    task_table::{Field, TaskTable},
    workflow::inbox_task,
};
use anyhow::Result;
use clap::Args;
use comfy_table::Color;
use console::style;
use durandal_core::CliMetaCommand;

use super::RFCUtil;

/// Daily inbox review.
///
/// Will also run the rfc_util subcommand.
#[derive(Args)]
pub struct Inbox;

impl CliMetaCommand for Inbox {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let mut client = TaskClient::new();
        client.filter("status:pending +in");

        // process all normal inbox items
        loop {
            // We need to refresh every time because of how ids change.
            // I suppose I could figure out how to update these without the ids,
            // but I'm trying to stick to "normal" cli operations
            client.refresh_tasks()?;
            let tasks = client.tasks();

            let cols = vec![Field::AnnotatedDescription];

            if let Some(task) = tasks.into_iter().next() {
                let mut table = TaskTable::new(&cols).description_color(Color::DarkYellow);
                table.add_row(&task);
                println!("\n\nThe next item is:\n");
                println!("{}\n", table);
                inbox_task(task.clone())?;
            } else {
                println!("{}", style("Your inbox is empty").yellow());
                break;
            }
        }

        // now handle rfcs that are potentially ready for review
        let rfc_util = RFCUtil;
        rfc_util.run(config)?;

        Ok(())
    }
}
