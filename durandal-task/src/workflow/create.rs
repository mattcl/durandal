use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::convert::{TryFrom, TryInto};
use task_hookrs::task::Task;

use crate::task::{projects, ActionCategory, Brainpower, Estimate, Project, TaskBuilder};

// Things that are required for a new task:
// * Action (required)
// * Context (required)
// * Project (optional)
// * Due Date (optional) * Brainpower (optional, default)
// * Time complexity (required)

#[derive(Debug, Clone)]
pub enum NewTask {
    ProjectInfo(Workflow<ProjectInfo>),
    Action(Workflow<Action>),
    Context(Workflow<Context>),
    Timing(Workflow<Timing>),
    Done(Workflow<Done>),
}

impl NewTask {
    pub fn new() -> Self {
        Self::ProjectInfo(Workflow {
            builder: TaskBuilder::new(),
            state: ProjectInfo,
        })
    }

    /// This is useful for adding a task to an existing project, namely in the
    /// workflow where we're determining what the next action should be for a
    /// project.
    pub fn for_project(project: Project) -> Self {
        let mut builder = TaskBuilder::new();
        builder.with_project(project);

        Self::Action(Workflow {
            builder,
            state: Action,
        })
    }

    pub fn step(self) -> Result<Self> {
        match self {
            Self::ProjectInfo(workflow) => workflow.step(),
            Self::Action(workflow) => workflow.step(),
            Self::Context(workflow) => workflow.step(),
            Self::Timing(workflow) => workflow.step(),
            _ => Err(anyhow::anyhow!(
                "Attempted to prompt for a terminal state {:?}",
                self
            )),
        }
    }

    pub fn terminated(&self) -> bool {
        match self {
            Self::Done(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workflow<S> {
    builder: TaskBuilder,
    pub state: S,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectInfo;

impl Workflow<ProjectInfo> {
    pub fn step(mut self) -> Result<NewTask> {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Is this part of a project?")
            .default(true)
            .interact()?
        {
            let mut projects = projects()?;

            // place a new project option at the front of the list
            projects.insert(0, "--New project--".into());

            let choice = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a project")
                .items(&projects)
                .default(0)
                .interact()?;

            let project: Project = if choice == 0 {
                // Prompt the user for a new project
                let s: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Project: ")
                    .interact_text()?;
                s.into()
            } else {
                projects[choice].clone()
            };

            self.builder.with_project(project);
        }

        Ok(NewTask::Action(self.into()))
    }
}

impl From<Workflow<ProjectInfo>> for Workflow<Action> {
    fn from(value: Workflow<ProjectInfo>) -> Self {
        Workflow {
            builder: value.builder,
            state: Action {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Action;

impl Workflow<Action> {
    pub fn step(mut self) -> Result<NewTask> {
        let desc: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("What is the action?")
            .interact_text()?;

        self.builder.with_description(desc);

        Ok(NewTask::Context(self.into()))
    }
}

impl From<Workflow<Action>> for Workflow<Context> {
    fn from(value: Workflow<Action>) -> Self {
        Workflow {
            builder: value.builder,
            state: Context {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Context;

impl Workflow<Context> {
    pub fn step(mut self) -> Result<NewTask> {
        let choices = ActionCategory::list();

        let selected = loop {
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("What context(s) fit this task? (space to select/unselect)")
                .items(&choices)
                .interact()?;

            let selected: Vec<_> = selections.iter().map(|s| choices[*s]).collect();

            if !selected.is_empty() {
                break selected;
            }

            println!(
                "  {}  ",
                style("You must specify at least one context").red()
            );
        };

        self.builder.with_contexts(&selected);

        // brainpower
        let choices = Brainpower::list();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("How much brainpower will this take?")
            .items(&choices)
            .default(1)
            .interact()?;

        self.builder.with_brainpower(choices[selection]);

        // estimate in minutes
        let choices = Estimate::list();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Rough estimate for how long this task will take?")
            .default(0)
            .items(&choices)
            .interact()?;

        self.builder.with_estimate(choices[selection]);

        Ok(NewTask::Timing(self.into()))
    }
}

impl From<Workflow<Context>> for Workflow<Timing> {
    fn from(value: Workflow<Context>) -> Self {
        Workflow {
            builder: value.builder,
            state: Timing,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timing;

impl Workflow<Timing> {
    pub fn step(mut self) -> Result<NewTask> {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Is there a specific due date?")
            .default(false)
            .interact()?
        {
            let due: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("When is it due?")
                .interact_text()?;
            self.builder.with_due(due);
        }

        Ok(NewTask::Done(self.try_into()?))
    }
}

impl TryFrom<Workflow<Timing>> for Workflow<Done> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Timing>) -> Result<Self> {
        let task = value.builder.build()?;
        let msg = format!(
            "Added a new task with id {}",
            task.id()
                .expect("Expected id, since we already read it in once")
        );

        println!("    {}", style(msg).green());

        Ok(Workflow {
            builder: value.builder,
            state: Done { task },
        })
    }
}

#[derive(Debug, Clone)]
pub struct Done {
    pub task: Task,
}
