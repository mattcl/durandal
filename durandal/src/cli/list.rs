use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use clap::Args;
use durandal_core::CliCommand;

/// List the installed external subcommands
///
/// This will look for commands in the executable path prefixed with 'durandal-'.
#[derive(Args)]
pub struct List;

impl CliCommand for List {
    fn run(&self) -> anyhow::Result<()> {
        // this is the way cargo does it, which seems to make sense
        let prefix = "durandal-";
        let mut commands = BTreeSet::new();

        for dir in search_directories() {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                _ => continue,
            };

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let filename = match path.file_name().and_then(|s| s.to_str()) {
                    Some(filename) => filename,
                    _ => continue,
                };

                if !filename.starts_with(prefix) {
                    continue;
                }

                if is_executable(entry.path()) {
                    let name = filename[prefix.len()..].to_string();
                    commands.insert(name);
                }
            }
        }

        if !commands.is_empty() {
            println!("The following external subcommands were detected.");
            println!("Run `durandal SUBCOMMAND -h/--help` for more information.\n");
            for command in commands.iter() {
                println!("    {command}");
            }
        } else {
            println!("No external subcommands detected.");
        }

        Ok(())
    }
}

fn search_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(val) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&val));
    }
    dirs
}

// this is the way cargo does it, which seems to make sense
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;

    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
