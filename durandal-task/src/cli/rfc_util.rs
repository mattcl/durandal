use crate::task::{ActionCategory, Processable};
use anyhow::{bail, Context, Result};
use clap::Args;
use console::style;
use durandal_core::CliMetaCommand;
use linkify::LinkFinder;

use crate::{
    config::Config,
    task::{load_tasks, TaskBuilder},
};

/// Create rnr tasks for RFC tickets if RFC links are available
#[derive(Args)]
pub struct RFCUtil;

impl CliMetaCommand for RFCUtil {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let rfcs = load_tasks(&config.rfcs.filter).with_context(|| "Could not fetch rfcs")?;

        let finder = LinkFinder::new();

        let eligible: Vec<_> = rfcs
            .iter()
            .filter_map(|rfc| {
                if let Some(task_hookrs::uda::UDAValue::Str(desc)) =
                    rfc.uda().get("jiradescription")
                {
                    if let Some(link) = finder
                        .links(desc)
                        .find(|link| link.as_str().contains("notion"))
                    {
                        return Some((rfc, link.as_str()));
                    }
                }
                None
            })
            .collect();

        if !eligible.is_empty() {
            println!("\n{}", style("Eligible RFCs detected").yellow());

            for (rfc, notion_link) in eligible {
                let mut builder = TaskBuilder::new();
                let description = match rfc.uda().get("jirasummary") {
                    Some(task_hookrs::uda::UDAValue::Str(summary)) => {
                        format!("RFC Review: {}", summary)
                    }
                    _ => bail!("Failed to get jira summary from: {:?}", rfc.id()),
                };

                let newtask = builder
                    .with_description(&description)
                    .with_contexts(&[ActionCategory::Computer, ActionCategory::Work])
                    .with_tags(["rnr"])
                    .with_project(config.rfcs.rnr_task_project.clone().into())
                    .build()?;

                // remove the rfc_inbox tag from the original task
                rfc.remove_tags(&["rfc_inbox"])?;

                // proceed to annotate the rnr task with relevant links
                let jira_link = match rfc.uda().get("jiraurl") {
                    Some(task_hookrs::uda::UDAValue::Str(link)) => link,
                    _ => bail!("Failed to get jira url from: {:?}", rfc.id()),
                };

                newtask.annotate(jira_link)?;
                newtask.annotate(notion_link)?;

                println!(
                    "{}",
                    style(format!("Added rnr task for '{}'", description)).green()
                );
            }
        }

        Ok(())
    }
}
