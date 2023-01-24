#![deny(missing_docs, rust_2018_idioms, clippy::perf)]

//! Doem
//! Generic terminal-based TODO app.

use std::fmt::Display;
use std::io::{stdout, ErrorKind, Write};
use std::str::FromStr;

use crossterm::queue;
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};

enum Commands {
    Add(Todo),
    Remove(String),
    List,
}

/// Command for the app to run.
pub struct Command(Commands);

impl Command {
    /// Adds a TODO.
    pub fn add(title: String, content: String, urgency: String) -> Option<Self> {
        let urgency = match urgency.as_str() {
            "l" | "low" => Urgency::Low,
            "m" | "medium" => Urgency::Medium,
            "h" | "high" => Urgency::High,
            _ => return None,
        };

        Some(Self(Commands::Add(Todo {
            title,
            content,
            urgency,
        })))
    }
    /// Removes a TODO.
    pub fn remove(title: String) -> Self {
        Self(Commands::Remove(title))
    }
    /// Lists TODOs.
    pub fn list() -> Self {
        Self(Commands::List)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Urgency {
    Low,
    Medium,
    High,
}

impl FromStr for Urgency {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "Low" => Ok(Self::Low),
            "Medium" => Ok(Self::Medium),
            "High" => Ok(Self::High),
            _ => Err("Failed to convert `str` to `Urgency`".to_owned()),
        }
    }
}

impl Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

struct Todo {
    title: String,
    content: String,
    urgency: Urgency,
    // due_date: Date, // Maybe not
}

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}: {}", self.urgency, self.title, self.content)
    }
}

fn read_todos() -> Option<String> {
    let dir = format!("{}/TODO", home::home_dir()?.display());
    let file = match std::fs::read_to_string(&dir) {
        Ok(file) => file,
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    if let Err(err) = std::fs::write(dir, "") {
                        eprintln!("Failed to create TODO file: {:?}", err)
                    }
                }
                other_err => {
                    eprintln!("Failed to read TODO file: {:?}", other_err);
                }
            }
            return None;
        }
    };
    Some(file)
}

/// Gets and parses TODO.
fn get_todos() -> Option<Vec<Todo>> {
    let mut todos = Vec::new();
    for line in read_todos()?.lines() {
        let (urgency, line) = line.split_once('|').expect("Incorrect TODO syntax");
        let (title, content) = line.split_once(": ").expect("Incorrect TODO syntax");
        let urgency = Urgency::from_str(urgency).expect("Incorrect TODO syntax");
        todos.push(Todo {
            title: title.to_owned(),
            content: content.to_owned(),
            urgency,
        });
    }
    Some(todos)
}

fn print_todo(todo: &Todo) -> crossterm::Result<()> {
    let mut stdout = stdout();

    let color = match todo.urgency {
        Urgency::Low => Color::Green,
        Urgency::Medium => Color::Yellow,
        Urgency::High => Color::Red,
    };

    queue!(
        stdout,
        Print("  "),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(color),
        Print(format!("[{}]", &todo.title)),
        SetAttribute(Attribute::Reset),
        Print(format!(" {}\r\n", todo.content))
    )?;

    stdout.flush()
}

fn save_todos(todos: Vec<Todo>) -> crossterm::Result<()> {
    if let Some(dir) = home::home_dir() {
        let dir = format!("{}/TODO", dir.display());
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&dir)?;
        file.write_all(
            todos
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes(),
        )?;
        file.flush()?;
    } else {
        eprintln!("Failed to find home directory");
    }

    Ok(())
}

/// Runs the program.
pub fn run(command: Command) -> crossterm::Result<()> {
    let mut todos = get_todos().unwrap_or_default();
    match command.0 {
        Commands::Add(todo) => {
            todos.push(todo);
            save_todos(todos)?;
        }
        Commands::Remove(title) => {
            let (rm_idx, _) = if let Some(idx) = todos
                .iter()
                .enumerate()
                .find(|(_, todo)| todo.title == title)
            {
                idx
            } else {
                eprintln!("Found no TODO with title: {title}");
                return Ok(());
            };

            todos.remove(rm_idx);
            save_todos(todos)?;
        }
        Commands::List => {
            for todo in todos {
                print_todo(&todo)?;
            }
        }
    }

    Ok(())
}
