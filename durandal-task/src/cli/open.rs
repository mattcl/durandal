use std::collections::HashSet;

use anyhow::{anyhow, Result};
use clap::Args;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use durandal_core::CliMetaCommand;
use linkify::LinkFinder;
use task_hookrs::{task::Task, uda::UDAValue};

use crate::{cli::make_iou_client, config::Config, task::active_task, task_table::UDA};

/// Open the ACTIVE task in a browser, if it can be.
///
/// Only supports Jira and Github issues for the time being.
#[derive(Args)]
pub struct Open;

impl CliMetaCommand for Open {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        let task = active_task()?.ok_or_else(|| anyhow!("Can only open an active task"))?;

        let mut links = get_links(&task);

        if links.is_empty() {
            println!("{}", style("No links detected in active task").yellow());
            return Ok(());
        }

        let iou_client = make_iou_client(&config)?;

        let choices: Vec<_> = links.drain().collect();

        // if there's only one link, just open the thing. Otherwise, prompt
        if links.len() == 1 {
            iou_client.open(&choices[0])?;
        } else {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Which URL to open?")
                .default(0)
                .items(&choices)
                .interact()?;

            // we "know" this access is safe
            iou_client.open(&choices[selection])?;
        }

        Ok(())
    }
}

fn get_links(task: &Task) -> HashSet<String> {
    let mut links = HashSet::new();
    let udas = task.uda();

    // TODO: figure out where to actually store UDAs like jiraurl - MCL - 2022-03-17
    if let Some(UDAValue::Str(url)) = udas.get("jiraurl") {
        links.insert(url.clone());
    }

    if let Some(UDAValue::Str(url)) = udas.get(UDA::GithubUrl.uda_key()) {
        links.insert(url.clone());
    }

    // now find links in annotations
    if let Some(annotations) = task.annotations() {
        let finder = LinkFinder::new();

        for annotation in annotations {
            finder.links(annotation.description()).for_each(|link| {
                links.insert(link.as_str().to_string());
            });
        }
    }

    links
}
