use crate::iou_client::IouClient;
use crate::task::Project;
use crate::workflow::create::NewTask;
use crate::workflow::inbox::InboxItem;
use crate::workflow::next::NextTask;
use crate::workflow::pr::Pr;
use anyhow::Result;
use task_hookrs::task::Task;

mod create;
mod inbox;
mod next;
mod pr;

pub trait StatefulEnum {
    type Output;
    fn step(self) -> Result<Self::Output>;
    fn terminated(&self) -> bool;
}

pub fn inbox_task(task: Task) -> Result<()> {
    let mut item = InboxItem::new(task);
    loop {
        item = item.step()?;
        if item.terminated() {
            break;
        }
    }
    Ok(())
}

pub fn new_task() -> Result<Task> {
    let mut workflow = NewTask::new();
    loop {
        workflow = workflow.step()?;

        match workflow {
            NewTask::Done(wf) => return Ok(wf.state.task),
            _ => {}
        }
    }
}

pub fn add_to_project(project: Project) -> Result<Task> {
    let mut workflow = NewTask::for_project(project);
    loop {
        workflow = workflow.step()?;

        match workflow {
            NewTask::Done(wf) => return Ok(wf.state.task),
            _ => {}
        }
    }
}

pub fn set_next_task(project: Project) -> Result<()> {
    let mut workflow = NextTask::new(project);
    loop {
        workflow = workflow.step()?;
        if workflow.terminated() {
            break;
        }
    }
    Ok(())
}

pub fn force_next_task(project: Project) -> Result<()> {
    let mut workflow = NextTask::with_force(project);
    loop {
        workflow = workflow.step()?;
        if workflow.terminated() {
            break;
        }
    }
    Ok(())
}

pub fn process_pr(task: Task, iou_client: &IouClient) -> Result<()> {
    let mut workflow = Pr::new(task, iou_client);
    loop {
        workflow = workflow.step()?;
        if workflow.terminated() {
            break;
        }
    }
    Ok(())
}
