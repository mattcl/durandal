use std::collections::HashSet;
use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::process::Command;

use anyhow::{bail, Context, Result};
use task_hookrs::import::import;
use task_hookrs::status::TaskStatus;
use task_hookrs::task::Task;

use crate::parser::new_task_parser;

/// This trait is just a convenient way to add some functionality to the Task
/// objects provided by `task_hookrs`
pub trait Processable {
    fn execute<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S> + Debug + Clone,
        S: AsRef<OsStr>;

    fn finish(&self) -> Result<()> {
        self.execute(["done"])
            .with_context(|| "Could not finish task")
    }

    fn delete(&self) -> Result<()> {
        self.execute(["delete"])
            .with_context(|| "Could not delete task")
    }

    fn annotate(&self, msg: &str) -> Result<()> {
        self.execute(["annotate", msg])
            .with_context(|| "Could not annotate task")
    }

    fn begin(&self) -> Result<()> {
        self.execute(["start"])
            .with_context(|| "Could not start task")
    }

    fn stop(&self) -> Result<()> {
        self.execute(["stop"])
            .with_context(|| "Could not stop task")
    }

    fn tickle(&self, wait: &str) -> Result<()> {
        self.execute(["modify", "+tickle", &format!("wait:{}", wait)])
            .with_context(|| "Could not add task to tickler")
    }

    fn someday(&self) -> Result<()> {
        self.execute(["modify", "-in", "-@home", "-@work", "proj:maybe"])
            .with_context(|| "Could not move task to maybe")
    }

    fn reference(&self) -> Result<()> {
        self.execute(["modify", "-in", "-@home", "-@work", "+reference"])
            .with_context(|| "Could not move task to be referenced")
    }

    fn add_tags(&self, tags: &[&str]) -> Result<()> {
        let mut args = vec!["modify".into()];
        for tag in tags.iter() {
            args.push(format!("+{}", tag));
        }
        self.execute(args)
            .with_context(|| format!("Could not add tags {:?}", tags))
    }

    fn remove_tags(&self, tags: &[&str]) -> Result<()> {
        let mut args = vec!["modify".into()];
        for tag in tags.iter() {
            args.push(format!("-{}", tag));
        }
        self.execute(args)
            .with_context(|| format!("Could not remove tags {:?}", tags))
    }

    fn set_resumable(&self, task: &Task) -> Result<()> {
        self.annotate(&format!("DTR:{}", task.uuid().to_string()))
            .with_context(|| "could not annotate task")
    }

    fn task_to_resume(&self) -> Result<Option<Task>>;

    /// Creates a follow-up task annotated with the description from the current
    /// task.
    ///
    /// `wait` can be any date format that taskwarrior accepts.
    fn follow_up(&self, msg: &str, wait: &str) -> Result<Task>;

    fn has_tag(&self, tag: &str) -> bool;

    fn is_next(&self) -> bool;

    fn annotated_description(&self) -> String;
}

impl Processable for Task {
    fn execute<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S> + Debug + Clone,
        S: AsRef<OsStr>,
    {
        if let Some(id) = self.id() {
            let output = Command::new("task")
                .arg("rc.confirmation=off")
                .arg(format!("{}", id))
                .args(args.clone())
                .output()
                .with_context(|| format!("Failed to execute command for task {}", id))?;

            if !output.status.success() {
                bail!(
                    "Command with args {:?} for task {} did not succeed with output {:?}",
                    args,
                    id,
                    output
                );
            }

            Ok(())
        } else {
            bail!("Task did not have an id! {:?}", self)
        }
    }

    fn task_to_resume(&self) -> Result<Option<Task>> {
        if let Some(annotations) = self.annotations() {
            if let Some(uuid) = annotations
                .iter()
                .rev()
                .find_map(|e| e.description().strip_prefix("DTR:"))
            {
                let task = load_task_from_uuid(uuid)?;
                // if we have an id, we're still not completed
                if task.id().is_some() {
                    return Ok(Some(task));
                }
            }
        }

        Ok(None)
    }

    fn follow_up(&self, msg: &str, wait: &str) -> Result<Task> {
        let task = TaskBuilder::new()
            .with_tags(["in", "tickle"])
            .with_wait(wait)
            .with_contexts(&[ActionCategory::Work, ActionCategory::Home])
            .with_description(msg)
            .build()
            .with_context(|| "Failed to create follow-up task")?;

        task.annotate(&format!("follow up from {}", self.description()))?;

        Ok(task)
    }

    fn has_tag(&self, tag: &str) -> bool {
        if let Some(tags) = self.tags() {
            return tags.contains(&tag.into());
        }

        false
    }

    fn is_next(&self) -> bool {
        // Completed tasks can never be next according to what we're using this
        // for
        match self.status() {
            TaskStatus::Completed => return false,
            _ => {}
        }

        self.has_tag("next")
    }

    fn annotated_description(&self) -> String {
        let mut s = self.description().clone();
        if let Some(annotations) = self.annotations() {
            for a in annotations {
                s += &format!("\n    {} {}", a.entry().format("%F"), a.description());
            }
        }
        s
    }
}

fn load_task(id: u64) -> Result<Task> {
    load_tasks(&format!("{}", id))?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not find task with id {}", id))
}

fn load_task_from_uuid(uuid: &str) -> Result<Task> {
    load_tasks(&format!("{}", uuid))?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not find task with uuid {}", uuid))
}

pub fn load_tasks(filter: &str) -> Result<Vec<Task>> {
    let mut task = Command::new("task");
    task.arg("rc.json.array=on");
    task.arg("rc.confirmation=off");

    if let Some(cmd) = shlex::split(filter) {
        for s in cmd {
            task.arg(&s);
        }
    }

    task.arg("export");

    let output = task.output()?;

    if !output.status.success() {
        bail!("Failed to load tasks: {:?}", output);
    }

    import(String::from_utf8_lossy(&output.stdout).as_bytes()).or_else(|e| {
        Err(anyhow::anyhow!(
            "Could not load from disk {:?}, {:?}",
            e,
            output
        ))
    })
}

pub fn active_tasks() -> Result<Vec<Task>> {
    let current_task_filter = "+ACTIVE";

    load_tasks(&current_task_filter).with_context(|| "Could not fetch current tasks")
}

pub fn active_task() -> Result<Option<Task>> {
    Ok(active_tasks()?.first().cloned())
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Project(String);

impl Project {
    pub fn tasks(&self) -> Result<Vec<Task>> {
        let filter = format!("proj:{} status:pending", self);
        load_tasks(&filter)
    }
}

impl Deref for Project {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Project {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<Project> for String {
    fn from(p: Project) -> Self {
        p.0
    }
}

impl From<String> for Project {
    fn from(s: String) -> Self {
        Project(s)
    }
}

impl From<&str> for Project {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

pub fn projects() -> Result<Vec<Project>> {
    let excluding = HashSet::new();
    projects_excluding(&excluding)
}

pub fn projects_excluding(exclude: &HashSet<String>) -> Result<Vec<Project>> {
    let output = Command::new("task")
        .arg("_projects")
        .output()
        .with_context(|| "Attempting to read projects")?;

    if !output.status.success() {
        bail!("Could not list projects: {:?}", output);
    }

    let out = String::from_utf8(output.stdout)?;

    Ok(out
        .split("\n")
        .into_iter()
        // exclude these special projects
        .filter(|s| !exclude.contains(*s))
        .map(|s| Project(s.to_string()))
        .collect())
}

#[derive(Debug, Clone, Default)]
pub struct TaskClient {
    tasks: Vec<Task>,
    filter: String,
}

impl TaskClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filter(&mut self, filter: &str) -> &mut Self {
        self.filter = filter.to_string();
        self
    }

    pub fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = load_tasks(&self.filter)?;
        Ok(())
    }
}

/// So I'm aware that the task-hookrs lib provides a builder, but since I'm
/// going to be interacting via the command line, I'm going to just make my
/// own.
#[derive(Debug, Clone)]
pub struct TaskBuilder {
    description: String,
    context: Vec<ActionCategory>,
    tags: Vec<String>,
    project: Option<Project>,
    wait: Option<String>,
    due: Option<String>,
    estimate: Estimate,
    brainpower: Brainpower,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            description: String::new(),
            context: Vec::new(),
            tags: Vec::new(),
            project: None,
            wait: None,
            due: None,
            estimate: Estimate::Small,
            brainpower: Brainpower::Medium,
        }
    }

    pub fn with_description<S: AsRef<str>>(&mut self, desc: S) -> &mut Self {
        self.description = desc.as_ref().into();
        self
    }

    pub fn with_contexts(&mut self, contexts: &[ActionCategory]) -> &mut Self {
        self.context.extend_from_slice(contexts);
        self
    }

    pub fn with_project(&mut self, project: Project) -> &mut Self {
        self.project = Some(project);
        self
    }

    pub fn with_tag<S: AsRef<str>>(&mut self, tag: S) -> &mut Self {
        self.tags.push(tag.as_ref().into());
        self
    }

    pub fn with_tags<I, S>(&mut self, tags: I) -> &mut Self
    where
        I: IntoIterator<Item = S> + Debug + Clone,
        S: AsRef<str>,
    {
        for t in tags {
            self.with_tag(t);
        }
        self
    }

    pub fn with_wait<S: AsRef<str>>(&mut self, wait: S) -> &mut Self {
        self.wait = Some(wait.as_ref().into());
        self
    }

    pub fn with_due<S: AsRef<str>>(&mut self, wait: S) -> &mut Self {
        self.due = Some(wait.as_ref().into());
        self
    }

    pub fn with_estimate(&mut self, estimate: Estimate) -> &mut Self {
        self.estimate = estimate;
        self
    }

    pub fn with_brainpower(&mut self, brainpower: Brainpower) -> &mut Self {
        self.brainpower = brainpower;
        self
    }

    pub fn build(&self) -> Result<Task> {
        let output = Command::new("task")
            .args(self.args())
            .output()
            .with_context(|| "Attempting to create a task via TaskBuilder")?;

        if !output.status.success() {
            bail!("Failed to create task: {:?}", output);
        }

        let out = String::from_utf8(output.clone().stdout)?;
        let id = new_task_parser(&out)
            .with_context(|| format!("Attempting to read newly created command id {:?}", output))?;
        load_task(id)
    }

    pub fn args(&self) -> Vec<String> {
        let mut args = vec![
            "add".into(),
            format!("brain:{}", self.brainpower.uda()),
            format!("est:{}", u64::from(self.estimate)),
        ];

        if let Some(ref proj) = self.project {
            args.push(format!("proj:{}", proj));
        }

        for c in &self.context {
            args.push(format!("+{}", c.tag()))
        }

        for t in &self.tags {
            args.push(format!("+{}", t))
        }

        if let Some(ref wait) = self.wait {
            args.push(format!("wait:{}", wait));
        }

        if let Some(ref due) = self.due {
            args.push(format!("due:{}", due));
        }

        args.push(self.description.clone());

        args
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActionCategory {
    Agenda,
    Anywhere,
    Computer,
    Errands,
    Home,
    Phone,
    ReadAndReview,
    Work,
}

impl ActionCategory {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Agenda,
            Self::Anywhere,
            Self::Computer,
            Self::Errands,
            Self::Home,
            Self::Phone,
            Self::ReadAndReview,
            Self::Work,
        ]
    }

    pub fn tag(&self) -> &str {
        match self {
            Self::Agenda => "@agenda",
            Self::Anywhere => "@anywhere",
            Self::Computer => "@computer",
            Self::Errands => "@errands",
            Self::Home => "@home",
            Self::Phone => "@phone",
            Self::ReadAndReview => "@rnr",
            Self::Work => "@work",
        }
    }
}

impl fmt::Display for ActionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Agenda => write!(f, "Agenda"),
            Self::Anywhere => write!(f, "Anywhere"),
            Self::Computer => write!(f, "Computer"),
            Self::Errands => write!(f, "Errands"),
            Self::Home => write!(f, "Home"),
            Self::Phone => write!(f, "Phone"),
            Self::ReadAndReview => write!(f, "ReadAndReview"),
            Self::Work => write!(f, "Work"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Brainpower {
    Low,
    Medium,
    High,
}

impl Brainpower {
    pub fn list() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High]
    }

    pub fn uda(&self) -> &str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::High => "H",
        }
    }
}

impl fmt::Display for Brainpower {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum Estimate {
    Small = 30,
    Medium = 360,
    Large = 1440,
    XLarge = 2880,
    Unknown = 9999,
}

impl Estimate {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Small,
            Self::Medium,
            Self::Large,
            Self::XLarge,
            Self::Unknown,
        ]
    }
}

impl fmt::Display for Estimate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small => write!(f, "Thirty minutes"),
            Self::Medium => write!(f, "Six hours"),
            Self::Large => write!(f, "One day"),
            Self::XLarge => write!(f, "Two days"),
            Self::Unknown => write!(f, "More than two days (not well understood)"),
        }
    }
}

impl From<Estimate> for u64 {
    fn from(e: Estimate) -> Self {
        e as u64
    }
}
