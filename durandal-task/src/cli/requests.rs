use crate::{config::Config, task::load_tasks, workflow::process_pr};
use anyhow::{Context, Result};
use clap::Args;
use durandal_core::CliMetaCommand;

use super::make_iou_client;

/// Process all requests.
#[derive(Args)]
pub struct Requests;

impl CliMetaCommand for Requests {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let iou_client = make_iou_client(&config)?;

        let pr_filter = "status:pending proj:hum.bw 'githubuser!~mattcl' \"(bw)PR\"";
        let prs = load_tasks(&pr_filter).with_context(|| "Could not fetch pull requests")?;

        for task in &prs {
            process_pr(task.clone(), &iou_client)?;
        }

        Ok(())
    }
}
