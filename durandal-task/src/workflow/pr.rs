use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use task_hookrs::task::Task;

use crate::{
    iou_client::IouClient,
    task::Processable,
    task_table::{Field, TaskDetail, UDA},
};

use super::StatefulEnum;

#[derive(Debug, Clone)]
pub enum Pr {
    Starting(Workflow<Starting>),
    Processing(Workflow<Processing>),
    Done(Workflow<Done>),
}

impl Pr {
    pub fn new(task: Task, iou_client: &IouClient) -> Self {
        Self::Starting(Workflow::new(task, iou_client.clone()))
    }
}

impl StatefulEnum for Pr {
    type Output = Self;

    fn step(self) -> Result<Self::Output> {
        match self {
            Self::Starting(machine) => machine.step(),
            Self::Processing(machine) => machine.step(),
            _ => Err(anyhow::anyhow!(
                "Attempted to prompt for a terminal state {:?}",
                self
            )),
        }
    }

    fn terminated(&self) -> bool {
        match self {
            Self::Done(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workflow<S> {
    iou_client: IouClient,
    task: Task,
    _state: S,
}

#[derive(Debug, Clone)]
pub struct Starting;

impl Workflow<Starting> {
    pub fn new(task: Task, iou_client: IouClient) -> Self {
        Self {
            iou_client,
            task,
            _state: Starting {},
        }
    }

    pub fn step(self) -> Result<Pr> {
        println!("{}", style("Next PR:").cyan());
        let mut desc = TaskDetail::new(&self.task);
        desc.add_row(&Field::ID);
        desc.add_rows(&[
            UDA::GithubTitle,
            UDA::GithubUser,
            UDA::GithubState,
            UDA::GithubUrl,
            UDA::GithubBody,
        ]);
        desc.add_row(&Field::Annotations);

        println!("{}", desc.output());

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Open in browser?")
            .default(true)
            .interact()?
        {
            let url = UDA::GithubUrl.get_raw_value(&self.task);
            if url.is_empty() {
                anyhow::bail!(format!(
                    "Task is missing the github url UDA: {:?}",
                    self.task
                ));
            }
            self.iou_client.open(&url)?;
        }

        Ok(Pr::Processing(self.into()))
    }
}

impl From<Workflow<Starting>> for Workflow<Processing> {
    fn from(value: Workflow<Starting>) -> Self {
        Self {
            iou_client: value.iou_client,
            task: value.task,
            _state: Processing {},
        }
    }
}

#[derive(Debug, Clone)]
pub struct Processing;

impl Workflow<Processing> {
    pub fn step(self) -> Result<Pr> {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to hide this PR until later?")
            .default(false)
            .interact()?
        {
            let wait: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("When would you like to see this PR again? (any valid 'wait:' value)")
                .default("+1d".into())
                .interact_text()?;

            self.task.tickle(&wait)?;
        }

        Ok(Pr::Done(self.into()))
    }
}

impl From<Workflow<Processing>> for Workflow<Done> {
    fn from(value: Workflow<Processing>) -> Self {
        Self {
            iou_client: value.iou_client,
            task: value.task,
            _state: Done {},
        }
    }
}

#[derive(Debug, Clone)]
pub struct Done;
