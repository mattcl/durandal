use anyhow::{self, bail, Context, Result};
use clap::{Parser, Subcommand};
use durandal_core::external::ExternalCommand;

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

#[derive(Subcommand)]
pub(crate) enum Commands {
    #[clap(alias = "commands")]
    List(List),

    #[clap(external_subcommand)]
    External(Vec<String>),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.run(),
            // This match arm terminates the current process because `run`
            // will exit
            Self::External(args) => {
                if args.is_empty() {
                    bail!("Unexpected empty external subcommand arg vector");
                }

                ExternalCommand::new()
                    .prefix("durandal")
                    .name(&args[0])
                    .args(&args[1..])
                    .build()?
                    .run()
            }
        }
    }
}
