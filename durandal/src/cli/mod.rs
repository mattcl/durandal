use anyhow::Result;
use clap::{Parser, Subcommand};
use durandal_core::CliDispatch;

mod list;

use self::list::List;

#[derive(Parser)]
#[clap(name = "durandal", author, version, about)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run() -> Result<()> {
        Self::parse().command.run()
    }
}

#[derive(Subcommand, CliDispatch)]
pub(crate) enum Commands {
    #[clap(alias = "commands")]
    List(List),

    #[clap(external_subcommand)]
    External(Vec<String>),
}
