use std::convert::{TryFrom, TryInto};
use std::fmt;

use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use dialoguer::{Input, Select};
use task_hookrs::task::Task;

use crate::task::Processable;

use super::create::NewTask;

#[derive(Debug, Clone)]
pub enum InboxItem {
    Starting(Workflow<Starting>),
    Inactioning(Workflow<Inactioning>),
    Incubating(Workflow<Incubating>),
    Incubated(Workflow<Incubated>),
    Referenced(Workflow<Referenced>),
    Actioning(Workflow<Actioning>),
    Delegating(Workflow<Delegating>),
    Delegated(Workflow<Delegated>),
    Deferring(Workflow<Deferring>),
    Deferred(Workflow<Deferred>),
    Finished(Workflow<Finished>),
    Deleted(Workflow<Deleted>),
}

impl InboxItem {
    pub fn new(task: Task) -> Self {
        Self::Starting(Workflow::new(task))
    }

    pub fn step(self) -> Result<Self> {
        match self {
            Self::Starting(machine) => machine.step(),
            Self::Inactioning(machine) => machine.step(),
            Self::Incubating(machine) => machine.step(),
            Self::Actioning(machine) => machine.step(),
            Self::Deferring(machine) => machine.step(),
            Self::Delegating(machine) => machine.step(),
            _ => Err(anyhow::anyhow!(
                "Attempted to prompt for a terminal state {:?}",
                self
            )),
        }
    }

    pub fn terminated(&self) -> bool {
        match self {
            Self::Incubated(_)
            | Self::Referenced(_)
            | Self::Delegated(_)
            | Self::Deferred(_)
            | Self::Finished(_)
            | Self::Deleted(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workflow<S> {
    task: Task,
    _state: S,
}

#[derive(Debug, Clone, Copy)]
pub struct Starting;

impl Workflow<Starting> {
    pub fn new(task: Task) -> Workflow<Starting> {
        Workflow {
            task,
            _state: Starting {},
        }
    }

    pub fn step(self) -> Result<InboxItem> {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Is this actionable (could you start work on this now)?")
            .default(true)
            .interact()?
        {
            Ok(InboxItem::Actioning(self.into()))
        } else {
            Ok(InboxItem::Inactioning(self.into()))
        }
    }
}

impl From<Workflow<Starting>> for Workflow<Inactioning> {
    fn from(item: Workflow<Starting>) -> Self {
        Workflow {
            task: item.task,
            _state: Inactioning {},
        }
    }
}

impl From<Workflow<Starting>> for Workflow<Actioning> {
    fn from(value: Workflow<Starting>) -> Self {
        Workflow {
            task: value.task,
            _state: Actioning {},
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NonAction {
    Trash,
    Incubate,
    Reference,
}

impl NonAction {
    pub fn list() -> Vec<NonAction> {
        vec![Self::Trash, Self::Incubate, Self::Reference]
    }
}

impl fmt::Display for NonAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trash => write!(f, "Trash"),
            Self::Incubate => write!(f, "Incubate"),
            Self::Reference => write!(f, "Reference"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Inactioning;

impl Workflow<Inactioning> {
    pub fn step(self) -> Result<InboxItem> {
        let unactionable = NonAction::list();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&unactionable)
            .interact()?;

        Ok(match unactionable[selection] {
            NonAction::Trash => InboxItem::Deleted(self.try_into()?),
            NonAction::Incubate => InboxItem::Incubating(self.into()),
            NonAction::Reference => InboxItem::Referenced(self.try_into()?),
        })
    }
}

impl TryFrom<Workflow<Inactioning>> for Workflow<Deleted> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Inactioning>) -> Result<Self> {
        value.task.delete()?;
        println!("{}", style("    Task deleted").red());

        Ok(Workflow {
            task: value.task,
            _state: Deleted {},
        })
    }
}

impl From<Workflow<Inactioning>> for Workflow<Incubating> {
    fn from(value: Workflow<Inactioning>) -> Self {
        Workflow {
            task: value.task,
            _state: Incubating {},
        }
    }
}

impl TryFrom<Workflow<Inactioning>> for Workflow<Referenced> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Inactioning>) -> Result<Self> {
        value.task.reference()?;
        println!("{}", style("    Task filed for reference").green());

        Ok(Workflow {
            task: value.task,
            _state: Referenced {},
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Incubating;

impl Workflow<Incubating> {
    pub fn step(self) -> Result<InboxItem> {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to be reminded about this task later?")
            .default(true)
            .interact()?
        {
            // tickler
            let wait: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("When would you like to be reminded? (any valid 'wait:' value)")
                .default("+1d".into())
                .interact_text()?;

            self.task.tickle(&wait)?;
        } else {
            // someday/maybe
            self.task.someday()?;
        }

        Ok(InboxItem::Incubated(self.into()))
    }
}

impl From<Workflow<Incubating>> for Workflow<Incubated> {
    fn from(value: Workflow<Incubating>) -> Self {
        println!("{}", style("    Task incubated").green());
        Workflow {
            task: value.task,
            _state: Incubated {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Incubated;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Action {
    Do,
    Defer,
    Delegate,
}

impl Action {
    pub fn list() -> Vec<Action> {
        vec![Self::Do, Self::Defer, Self::Delegate]
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Do => write!(f, "Do it"),
            Self::Defer => write!(f, "Defer"),
            Self::Delegate => write!(f, "Delegate"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Actioning;

impl Workflow<Actioning> {
    pub fn step(self) -> Result<InboxItem> {
        let choices = Action::list();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&choices)
            .interact()?;

        Ok(match choices[selection] {
            Action::Do => {
                // So this is a little dumb, but if you've decided to do something
                // in the inbox, we need to wait for it to be done and there should
                // be no normal way to get around that.
                //
                // Since this is so simple, I decided against having a "Doing" state
                // which would have just increased the complexity
                loop {
                    if Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Is it done?")
                        .default(true)
                        .interact()?
                    {
                        break InboxItem::Finished(self.try_into()?);
                    }
                }
            }
            Action::Defer => InboxItem::Deferring(self.into()),
            Action::Delegate => InboxItem::Delegating(self.into()),
        })
    }
}

impl TryFrom<Workflow<Actioning>> for Workflow<Finished> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Actioning>) -> Result<Self> {
        value.task.finish()?;
        println!("{}", style("    Task finished").green());

        Ok(Workflow {
            task: value.task,
            _state: Finished {},
        })
    }
}

impl From<Workflow<Actioning>> for Workflow<Deferring> {
    fn from(value: Workflow<Actioning>) -> Self {
        Workflow {
            task: value.task,
            _state: Deferring {},
        }
    }
}

impl From<Workflow<Actioning>> for Workflow<Delegating> {
    fn from(value: Workflow<Actioning>) -> Self {
        Workflow {
            task: value.task,
            _state: Delegating {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Deferring;

impl Workflow<Deferring> {
    pub fn step(self) -> Result<InboxItem> {
        let mut newtask = NewTask::new();

        loop {
            newtask = newtask
                .step()
                .with_context(|| "Attempting to create a task as part of the deferring step")?;

            if newtask.terminated() {
                break;
            }
        }

        Ok(InboxItem::Deferred(self.try_into()?))
    }
}

impl TryFrom<Workflow<Deferring>> for Workflow<Deferred> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Deferring>) -> Result<Self> {
        value.task.delete()?;
        println!("{}", style("    Task deferred (original deleted)").green());

        Ok(Workflow {
            task: value.task,
            _state: Deferred,
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Delegate {
    Slack,
    Email,
    Phone,
    Agenda,
}

impl Delegate {
    fn list() -> Vec<Delegate> {
        vec![Self::Slack, Self::Email, Self::Phone, Self::Agenda]
    }

    fn annotation(&self) -> &str {
        match self {
            Self::Slack => "Sent a slack message",
            Self::Email => "Sent an email",
            Self::Phone => "Texted or called",
            Self::Agenda => "Meant to bring it up in the next meeting",
        }
    }
}

impl fmt::Display for Delegate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Slack => write!(f, "Send a slack message to this person"),
            Self::Email => write!(f, "Send an email to this person"),
            Self::Phone => write!(f, "Text/Call this person"),
            Self::Agenda => write!(f, "Make a note for the next meeting with this person"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delegating;

impl Workflow<Delegating> {
    pub fn step(self) -> Result<InboxItem> {
        let choices = Delegate::list();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&choices)
            .interact()?;

        let msg: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like the reminder for this follow-up to be?")
            .interact()?;

        let wait: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("When should this follow-up appear in your inbox?")
            .interact()?;

        let follow_up = self.task.follow_up(&msg, &wait)?;

        // Annotate with what delegation option we chose for later reference
        follow_up.annotate(choices[selection].annotation())?;

        Ok(InboxItem::Delegated(self.try_into()?))
    }
}

impl TryFrom<Workflow<Delegating>> for Workflow<Delegated> {
    type Error = anyhow::Error;

    fn try_from(value: Workflow<Delegating>) -> Result<Self> {
        value.task.delete()?;
        println!("{}", style("    Task delegated (original deleted)").green());

        Ok(Workflow {
            task: value.task,
            _state: Delegated {},
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Referenced;

#[derive(Debug, Clone, Copy)]
pub struct Delegated;

#[derive(Debug, Clone, Copy)]
pub struct Deferred;

#[derive(Debug, Clone, Copy)]
pub struct Finished;

#[derive(Debug, Clone, Copy)]
pub struct Deleted;
