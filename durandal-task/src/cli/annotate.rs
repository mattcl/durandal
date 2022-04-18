use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Input};
use durandal_core::CliMetaCommand;

use crate::{
    config::Config,
    task::{active_tasks, Processable},
};

/// Annotate the ACTIVE task
///
/// Message can either be specified on the command line or via prompt.
#[derive(Args)]
pub struct Annotate {
    /// The annotation.
    ///
    /// If not provided, user will be prompted for message instead.
    #[clap(short, long)]
    message: Option<String>,
}

impl CliMetaCommand for Annotate {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        let candidates = active_tasks()?;

        let task = candidates
            .first()
            .ok_or_else(|| anyhow!("Can only annotate active task"))?;

        let msg = if let Some(ref msg) = self.message {
            msg.clone()
        } else {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Message?")
                .interact_text()?
        };

        task.annotate(&msg)?;

        Ok(())
    }
}
