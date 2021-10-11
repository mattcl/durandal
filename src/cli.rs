use std::{collections::HashSet, env};
use std::fs::read_dir;
use std::process::Command;

use anyhow::Result;
use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, ArgMatches,
    SubCommand,
};
use console::style;
use which::which;

pub fn cli() -> ! {
    let external_help = make_external_help();

    if let Err(e) = external_help {
        clap::Error::with_description(
            &format!("{:?}", e),
            clap::ErrorKind::InvalidValue, // just a generic kind, but it could be anything
        )
        .exit()
    }

    let external_help_text = external_help.unwrap().unwrap_or_default(); // this is safe now

    let mut app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Use -h for short descriptions and --help for more details.");

    if !external_help_text.is_empty() {
        app = app.after_help(external_help_text.as_str());
    }
    app = app
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::AllowExternalSubcommands)
        .subcommand(SubCommand::with_name("hello").about("prints hello message"));

    let matches = app.clone().get_matches();

    let res: Result<i32> = match matches.subcommand() {
        // It's important to explicitly match all defined subcommands, otherwise
        // we'll hit the catch-all at the bottom and attempt an external command
        // execution.
        ("hello", _hello_matches) => {
            println!("Hello cold world");
            Ok(0)
        }

        // if no subcommand was provided, just print the help message
        ("", _) => {
            app.print_help().unwrap();
            // so for whatever reason, there's no newline after print_help()
            println!();
            Ok(0)
        }

        // Attempt to execute the subcommand as an external program
        (ext_name, matches) => try_external(ext_name, matches.unwrap()),
    };

    match res {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            clap::Error::with_description(
                &format!("{:?}", e),
                clap::ErrorKind::InvalidValue, // just a generic kind, but it could be anything
            )
            .exit()
        }
    }
}

// This always exits the process, hence the type -> !
fn try_external(ext_name: &str, matches: &ArgMatches) -> ! {
    // if we find an external program of the form appname-subcommand,
    // execute that external program with any supplied arguments and exit
    // with that program's exit code
    let external = format!("{}-{}", crate_name!(), ext_name);
    if let Ok(_) = which(external.clone()) {
        // handle the situation where there are no additional arguments
        let args: Vec<&str> = if let Some(matches) = matches.values_of("") {
            matches.collect()
        } else {
            Vec::new()
        };

        let res = Command::new(external)
            .args(args)
            .status()
            .expect("Could not execute subcommand");

        std::process::exit(res.code().unwrap_or(0))
    } else {
        // If we couldn't find the executable we were expecting, let the
        // user know and exit
        clap::Error::with_description(
            &format!(
                "Unknown subcommand: '{}' Expected executable {} in path but was not found.",
                ext_name, external
            ),
            clap::ErrorKind::UnrecognizedSubcommand,
        )
        .exit()
    }
}

fn find_external_commands() -> Result<HashSet<String>>{
    let mut cmds = HashSet::new();
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if path.is_dir() {
                for entry in read_dir(path)? {
                    let p = entry?.path();
                    if p.is_file() {
                        let name = p.file_name().unwrap_or_default().to_string_lossy();
                        let prefix = format!("{}-", crate_name!());
                        if name.starts_with(&prefix) {
                            cmds.insert(name.strip_prefix(&prefix).unwrap_or_default().to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(cmds)
}

fn make_external_help() -> Result<Option<String>> {
    let external_commands = find_external_commands()?;
    if external_commands.is_empty() {
        return Ok(None);
    }

    let mut msg = String::from("The following external subcommands are available.\n");
    msg += &format!("Run `{} SUBCOMMAND -h/--help` for additional information\n", crate_name!());
    msg += &format!("{}", style("\nEXTERNAL SUBCOMMANDS:\n").yellow());

    for cmd in external_commands {
        msg += &format!("    {}\n", style(cmd).green());
    }

    Ok(Some(msg))
}
