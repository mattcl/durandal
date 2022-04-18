use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use task_hookrs::task::Task;

use crate::{
    task::{Processable, Project},
    workflow::add_to_project,
};

#[derive(Debug, Clone)]
pub enum NextTask {
    Checking(Workflow<Checking>),
    Picking(Workflow<Picking>),
    Done(Workflow<Done>),
}

impl NextTask {
    pub fn new(project: Project) -> Self {
        Self::Checking(Workflow {
            project,
            state: Checking::default(),
        })
    }

    pub fn with_force(project: Project) -> Self {
        Self::Checking(Workflow {
            project,
            state: Checking { force: true },
        })
    }

    pub fn step(self) -> Result<Self> {
        match self {
            Self::Checking(workflow) => workflow.step(),
            Self::Picking(workflow) => workflow.step(),
            _ => Err(anyhow::anyhow!(
                "Attempted to step for a terminal state {:?}",
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
    project: Project,
    state: S,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Checking {
    force: bool,
}

impl Workflow<Checking> {
    pub fn step(self) -> Result<NextTask> {
        let tasks = self.project.tasks()?;
        let target_tag = String::from("next");
        let cur_tasks: Vec<_> = tasks
            .iter()
            .filter_map(|tsk| {
                tsk.tags().and_then(|tags| {
                    if tags.contains(&target_tag) {
                        Some(tsk)
                    } else {
                        None
                    }
                })
            })
            .collect();

        if !cur_tasks.is_empty() {
            // there is a task with the next tag, so decide what do to
            if self.state.force {
                // clear the next tag for the next tasks before we continue
                for task in cur_tasks {
                    task.remove_tags(&["next"])?;
                }
            } else {
                // otherwise, we're done
                return Ok(NextTask::Done(self.into()));
            }
        }

        // there isn't one so we have to decide what to do in another step

        // this is a bit wonky but I'd prefer if step didn't have to take
        // mut self
        println!(
            "\n\nThe following project does not have a next task:\n -> {}\n",
            style(&self.project).yellow()
        );

        let mut nt: Workflow<Picking> = self.into();
        nt.state.tasks = tasks;

        Ok(NextTask::Picking(nt))
    }
}

impl From<Workflow<Checking>> for Workflow<Done> {
    fn from(value: Workflow<Checking>) -> Self {
        Workflow {
            project: value.project,
            state: Done,
        }
    }
}

impl From<Workflow<Checking>> for Workflow<Picking> {
    fn from(value: Workflow<Checking>) -> Self {
        Workflow {
            project: value.project,
            state: Picking { tasks: Vec::new() },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Picking {
    tasks: Vec<Task>,
}

impl Workflow<Picking> {
    pub fn step(self) -> Result<NextTask> {
        let mut choices: Vec<_> = self
            .state
            .tasks
            .iter()
            .map(|t| t.description().clone())
            .collect();

        choices.push("--New task--".into());

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which task should be next?")
            .items(&choices)
            .default(0)
            .interact()?;

        let task = match self.state.tasks.get(choice) {
            Some(task) => task.clone(),
            None => add_to_project(self.project.clone())?,
        };

        task.execute(["modify", "+next"])
            .with_context(|| "Failed attempting to modify task with +next")?;

        println!("    {}", style("Next task selected").green());

        Ok(NextTask::Done(self.into()))
    }
}

impl From<Workflow<Picking>> for Workflow<Done> {
    fn from(value: Workflow<Picking>) -> Self {
        Workflow {
            project: value.project,
            state: Done,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Done;
