use crate::config::Config;
use anyhow::Result;
use clap::Args;
use durandal_core::CliMetaCommand;

/// Weekly task review.
#[derive(Args)]
pub struct Weekly;

impl CliMetaCommand for Weekly {
    type Meta = Config;

    fn run(&self, _config: &Self::Meta) -> Result<()> {
        todo!()
    }
}
