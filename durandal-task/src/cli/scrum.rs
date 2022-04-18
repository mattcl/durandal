use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use chrono::prelude::*;
use clap::Args;
use comfy_table::Color;
use console::style;
use durandal_core::CliMetaCommand;
use handlebars::Handlebars;

use crate::{
    config::Config,
    task::load_tasks,
    task_table::{display_unique_table, Field},
};

/// Daily scrum summary.
#[derive(Args)]
pub struct Scrum {
    /// How many days ago to use as the cutoff.
    ///
    /// If not set, will rely on the day of the week to calculate the
    /// appropriate number of days.
    #[clap(short, long)]
    days: Option<u64>,
}

impl CliMetaCommand for Scrum {
    type Meta = Config;

    fn run(&self, config: &Self::Meta) -> Result<()> {
        // things that are important for scrum:
        // Tasks that are due today

        // We need to figure out which day we should use for our earliest tasks to
        // evaluate, so if it's not specified, use the current day of the week, and,
        // if it's a Monday, we need to look back to the previous Friday. Otherwise
        // just look at yesterday.
        let today = Local::now();
        let lower_bound = match self.days {
            Some(val) => format!("today-{val}d"),
            None => match today.weekday() {
                Weekday::Mon => String::from("today-3d"),
                Weekday::Sun => String::from("today-2d"),
                _ => String::from("yesterday"),
            },
        };

        let hbrs = Handlebars::new();
        let mut vars: HashMap<&str, &str> = HashMap::new();
        vars.insert("bound", &lower_bound);

        // let's load all the tasks at once so we don't show any output if we're
        // going to error

        let completed_filter = hbrs.render_template(&config.scrum.completed, &vars)?;
        let completed_tasks =
            load_tasks(&completed_filter).with_context(|| "Could not fetch completed tasks")?;

        let started_filter = hbrs.render_template(&config.scrum.in_progress, &vars)?;
        let started_tasks =
            load_tasks(&started_filter).with_context(|| "Could not fetch in progress tasks")?;

        let due_filter = hbrs.render_template(&config.scrum.due, &vars)?;
        let due_tasks = load_tasks(&due_filter).with_context(|| "Could not fetch due tasks")?;

        let modified_filter = hbrs.render_template(&config.scrum.modified, &vars)?;
        let modified_tasks =
            load_tasks(&modified_filter).with_context(|| "Could not fetch modified tasks")?;

        let followup_filter = hbrs.render_template(&config.scrum.waiting, &vars)?;
        let followup_tasks =
            load_tasks(&followup_filter).with_context(|| "Could not fetch follow-up tasks")?;

        let motd = format!(
            "Today is {}. Last scrum should have been {}\n",
            today.format("%A, %F"),
            lower_bound
        );
        println!("{}", style(motd).bold());

        let mut seen_uuids = HashSet::new();

        let standard_cols = vec![
            Field::ID,
            Field::Project,
            Field::Next,
            Field::AnnotatedDescription,
        ];

        // Tasks finished yesterday (or since Friday if Monday scrum)
        println!("{}", style("Tasks completed since last scrum:").cyan());
        display_unique_table(
            &completed_tasks,
            // showing the id or next for these would be pointless
            &vec![Field::Project, Field::Description],
            Color::Green,
            &mut seen_uuids,
        );

        // Tasks currently in progress
        println!("\n{}", style("In-progress tasks:").cyan());
        display_unique_table(&started_tasks, &standard_cols, Color::Blue, &mut seen_uuids);

        // Tasks due in the next 7 days or overdue
        println!("\n{}", style("Tasks due soon:").cyan());
        display_unique_table(
            &due_tasks,
            &vec![
                Field::ID,
                Field::Due,
                Field::Project,
                Field::Next,
                Field::AnnotatedDescription,
            ],
            Color::Red,
            &mut seen_uuids,
        );

        // Tasks waiting for others
        println!(
            "\n{}",
            style("Waiting tasks for the next five days:").cyan()
        );
        display_unique_table(
            &followup_tasks,
            &vec![
                Field::ID,
                Field::Waiting,
                Field::Project,
                Field::AnnotatedDescription,
            ],
            Color::Magenta,
            &mut seen_uuids,
        );

        // Tasks modified yesterday (or since Friday if Monday scrum)
        // this needs to be last since it's possible that it would catch tasks that
        // fit better in other sections, and, because we're only showing unique
        // tasks across any report, we don't want to not show the duplicate here
        // instead of elsewhere
        println!(
            "\n{}",
            style("Other tasks modified since last scrum:").cyan()
        );
        display_unique_table(
            &modified_tasks,
            &standard_cols,
            Color::DarkYellow,
            &mut seen_uuids,
        );

        Ok(())
    }
}
