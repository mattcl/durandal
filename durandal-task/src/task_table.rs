use std::{collections::HashSet, fmt};

use chrono::Local;
use comfy_table::{presets::NOTHING, Cell, Color, ContentArrangement, Table};
use console::style;
use task_hookrs::{task::Task, uda::UDAValue};

use crate::task::Processable;

#[derive(Debug)]
pub struct TaskTable<'a> {
    columns: &'a Vec<Field>,
    desc_color: Color,
    table: Table,
}

impl<'a> TaskTable<'a> {
    pub fn new(columns: &'a Vec<Field>) -> Self {
        let mut table = Table::new();
        table.load_preset(NOTHING);
        table.set_content_arrangement(ContentArrangement::Dynamic);

        Self {
            columns,
            desc_color: Color::White,
            table,
        }
    }

    pub fn description_color(mut self, color: Color) -> Self {
        self.desc_color = color;
        self
    }

    pub fn add_row(&mut self, task: &Task) {
        let mut row = Vec::new();
        for col in self.columns {
            row.push(
                Cell::new(col.get_value(task)).fg(col.get_color(task).unwrap_or(self.desc_color)),
            )
        }
        self.table.add_row(row);
    }
}

impl<'a> fmt::Display for TaskTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.table)
    }
}

#[derive(Debug)]
pub struct TaskDetail<'a> {
    task: &'a Task,
    table: Table,
}

impl<'a> TaskDetail<'a> {
    pub fn new(task: &'a Task) -> Self {
        let mut table = Table::new();
        table.load_preset(NOTHING);
        Self { task, table }
    }

    pub fn add_rows<T: TaskAttr>(&mut self, fields: &[T]) -> &mut Self {
        for f in fields {
            self.add_row(f);
        }
        self
    }

    pub fn add_row<T: TaskAttr>(&mut self, field: &T) -> &mut Self {
        self.custom_row(field.name(), &field.get_value(self.task));
        self
    }

    pub fn custom_row(&mut self, label: &str, value: &str) -> &mut Self {
        self.table.add_row(vec![
            Cell::new(label)
                .fg(Color::DarkGrey)
                .set_alignment(comfy_table::CellAlignment::Right),
            Cell::new(value),
        ]);
        self
    }

    pub fn output(&self) -> &Table {
        &self.table
    }
}

impl<'a> fmt::Display for TaskDetail<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.table)
    }
}

pub trait TaskAttr {
    fn name(&self) -> &str;
    fn get_value(&self, task: &Task) -> String;
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Description,
    AnnotatedDescription,
    Annotations,
    Due,
    ID,
    Next,
    Project,
    Waiting,
}

impl Field {
    pub fn get_color(&self, task: &Task) -> Option<Color> {
        match self {
            Self::Description => None,
            Self::AnnotatedDescription => None,
            Self::Annotations => None,
            Self::Due => {
                // there is a very unlikely edge here where, because we fetch
                // the current day twice, we *could* land on a date boundary,
                // but it's probably never going to happen, and, if it does
                // won't matter anyway
                let today = Local::now().naive_local().date();
                Some(match task.due() {
                    Some(ref date) if date.date() <= today => Color::DarkRed,
                    _ => Color::DarkGrey,
                })
            }
            Self::Next => Some(Color::Red),
            Self::Waiting => Some(match task.wait() {
                Some(_) => Color::DarkGrey,
                None => Color::Red,
            }),
            _ => Some(Color::DarkGrey),
        }
    }
}

impl TaskAttr for Field {
    fn get_value(&self, task: &Task) -> String {
        match self {
            Self::Description => task.description().into(),
            Self::AnnotatedDescription => task.annotated_description(),
            Self::Annotations => {
                let mut ann = vec![];
                if let Some(annotations) = task.annotations() {
                    for a in annotations {
                        ann.push(format!("{} {}", a.entry().format("%F"), a.description()));
                    }
                }
                ann.join("\n")
            }
            Self::Due => {
                let today = Local::now().naive_local().date();
                match task.due() {
                    Some(ref date) if date.date() == today => String::from("Today"),
                    Some(ref date) if date.date() < today => {
                        format!("Overdue ({})", date.format("%F"))
                    }
                    Some(ref date) => date.format("%F %a").to_string(),
                    None => String::from("None"),
                }
            }
            Self::ID => format!("{}", task.id().unwrap_or(0)),
            Self::Next => if task.is_next() { "N" } else { "" }.into(),
            Self::Project => task.project().unwrap_or(&String::new()).into(),
            Self::Waiting => match task.wait() {
                Some(date) => date.format("%F %a").to_string(),
                None => String::from("Ready"),
            },
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Description => "Description",
            Self::AnnotatedDescription => "Description",
            Self::Annotations => "Annotations",
            Self::Due => "Due",
            Self::ID => "ID",
            Self::Next => "Next label",
            Self::Project => "Project",
            Self::Waiting => "Waiting",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UDA {
    GithubTitle,
    GithubBody,
    GithubUser,
    GithubUrl,
    GithubState,
}

impl UDA {
    pub fn uda_key(&self) -> &str {
        match self {
            Self::GithubTitle => "githubtitle",
            Self::GithubBody => "githubbody",
            Self::GithubUser => "githubuser",
            Self::GithubUrl => "githuburl",
            Self::GithubState => "githubstate",
        }
    }

    pub fn get_raw_value(&self, task: &Task) -> String {
        // TODO: for now only support string values - MCL - 2021-10-24
        if let Some(UDAValue::Str(value)) = task.uda().get(self.uda_key()) {
            value.clone()
        } else {
            String::new()
        }
    }
}

impl TaskAttr for UDA {
    fn name(&self) -> &str {
        match self {
            Self::GithubTitle => "Title",
            Self::GithubBody => "Body",
            Self::GithubUser => "User",
            Self::GithubUrl => "URL",
            Self::GithubState => "State",
        }
    }

    fn get_value(&self, task: &Task) -> String {
        self.get_raw_value(task)
    }
}

/// Convenience method for displaying a vector of tasks as a table.
pub fn display_table(tasks: &Vec<Task>, columns: &Vec<Field>, description_color: Color) {
    let mut seen_uuids = HashSet::new();
    display_unique_table(tasks, columns, description_color, &mut seen_uuids);
}

/// Convenience method for displaying a vector of tasks as a table.
///
/// Unlike `display_table`, this allows you to specify the seen uuid cache to
/// use when determining uniqueness. Doing so lets you have a unique constraint
/// across a _set_ of tables.
pub fn display_unique_table(
    tasks: &Vec<Task>,
    columns: &Vec<Field>,
    description_color: Color,
    seen_uuids: &mut HashSet<uuid::Uuid>,
) {
    let tasks: Vec<_> = tasks
        .into_iter()
        .filter(|t| !seen_uuids.contains(t.uuid()))
        .collect();

    if tasks.is_empty() {
        println!("    {}", style("--none--").yellow());
    } else {
        let mut table = TaskTable::new(columns).description_color(description_color);

        for task in tasks {
            seen_uuids.insert(task.uuid().clone());
            table.add_row(task);
        }

        println!("{}", table);
    }
}
