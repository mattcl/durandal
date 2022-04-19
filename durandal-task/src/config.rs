use std::{collections::HashSet, ffi::OsStr, path::Path};

use anyhow::{bail, Result};
use config;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IouConfig {
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rfcs {
    pub filter: String,
    pub rnr_task_project: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Requests {
    pub filter: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scrum {
    pub completed: String,

    #[serde(default = "default_due")]
    pub due: String,

    #[serde(default = "default_in_progress")]
    pub in_progress: String,

    pub modified: String,

    #[serde(default = "default_waiting")]
    pub waiting: String,
}

fn default_due() -> String {
    String::from("-in +@work status:Pending and (+DUE or +OVERDUE)")
}

fn default_in_progress() -> String {
    String::from("-in +@work +ACTIVE")
}

fn default_waiting() -> String {
    String::from(
        "+@work -@home and ((+WAITING and wait.before:today+5d) or (+tickle and status:Pending))",
    )
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub iou: IouConfig,
    #[serde(default)]
    pub excluded_projects: HashSet<String>,
    pub rfcs: Rfcs,
    pub requests: Requests,
    pub scrum: Scrum,
}

pub fn default_location() -> Result<String> {
    if let Some(home) = dirs::home_dir() {
        let global_config = home.join(Path::new(OsStr::new("task.toml")));
        if let Some(path) = global_config.to_str() {
            return Ok(path.to_string());
        } else {
            bail!(format!(
                "Unable to load global config despite it existing at: {:?}",
                global_config
            ));
        }
    } else {
        bail!("Could not determine home directory");
    }
}

impl Config {
    pub fn new(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            bail!("Specified config path does not exist {}", path);
        }
        let mut raw = config::Config::default();
        raw.merge(config::File::with_name(path))?;

        Ok(raw.try_into()?)
    }
}
