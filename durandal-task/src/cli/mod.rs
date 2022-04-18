use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use durandal_core::CliMetaDispatch;

use crate::{
    config::{default_location, Config},
    iou_client::IouClient,
};

use self::annotate::Annotate;
use self::current::Current;
use self::done::Done;
use self::inbox::Inbox;
use self::interrupt::Interrupt;
use self::new::New;
use self::next::Next;
use self::open::Open;
use self::projects::Projects;
use self::replan::Replan;
use self::requests::Requests;
use self::rfc_util::RFCUtil;
use self::scrum::Scrum;
use self::stop::Stop;
use self::table::Table;
use self::weekly::Weekly;

mod annotate;
mod current;
mod done;
mod inbox;
mod interrupt;
mod new;
mod next;
mod open;
mod projects;
mod replan;
mod requests;
mod rfc_util;
mod scrum;
mod stop;
mod table;
mod weekly;

#[derive(Parser)]
#[clap(name = "durandal-task", author, version, about)]
pub(crate) struct Cli {
    /// Config file location
    #[clap(short = 'C', long)]
    pub config: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run() -> Result<()> {
        let app = Self::parse();

        let config_file = match app.config {
            Some(path) => path,
            None => default_location().with_context(|| "Could not load default config location")?,
        };

        let config = Config::new(&config_file)?;

        app.command.run(&config)
    }
}

#[derive(Subcommand, CliMetaDispatch)]
#[cli_meta(Config)]
pub(crate) enum Commands {
    #[clap(visible_alias = "comment")]
    Annotate(Annotate),
    Current(Current),
    #[clap(visible_alias = "finish")]
    Done(Done),
    Inbox(Inbox),
    #[clap(visible_alias = "int")]
    Interrupt(Interrupt),
    New(New),
    #[clap(visible_alias = "start")]
    Next(Next),
    Open(Open),
    #[clap(visible_alias = "proj")]
    Projects(Projects),
    Replan(Replan),
    #[clap(visible_alias = "req")]
    Requests(Requests),
    #[clap(name = "rfc_util")]
    RFCUtil(RFCUtil),
    Scrum(Scrum),
    Stop(Stop),
    Table(Table),
    Weekly(Weekly),
}

fn make_iou_client(config: &Config) -> Result<IouClient> {
    if config.iou.servers.is_empty() {
        bail!("Invalid configuration: IOU server list is empty");
    }

    if config.iou.servers.len() > 1 {
        let mut servers = Vec::new();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Wich configured IOU server would you like to use?")
            .items(&config.iou.servers)
            .default(0)
            .interact()?;

        servers.push(config.iou.servers[selection].clone());

        Ok(IouClient::new(servers))
    } else {
        Ok(IouClient::new(config.iou.servers.clone()))
    }
}
