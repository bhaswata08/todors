use std::env;
use std::fs;
use std::io::{prelude::*};
use std::path::Path;
use serde_derive::{Deserialize, Serialize};
use clap::{command, Parser};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Priority {
    Low,
    Medium,
    High,
    Urgent
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    item: String,
    status: bool,
    priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoSpace {
    name: String,
    todos: Vec<TodoItem>,
}

struct TodoManager {
    file_path: String,
    current_todo_spaces: Vec<TodoSpace>,
    current_space: Option<String>,
}

impl Priority {
    fn from_markdown(line:&str) -> Priority {
        if line.contains("{URGENT}") {
            Priority::Urgent
        } else if line.contains("{High}") {
            Priority::High
        } else if line.contains("{Medium}") {
            Priority::Medium
        } else if line.contains("{Low}") {
            Priority::Low
        } else {
            Priority::Medium
        }
    }
    fn to_markdown(&self) -> &str {
        match self {
            Priority::Urgent => "{URGENT}",
            Priority::High => "{HIGH}",
            Priority::Medium => "{MEDIUM}",
            Priority::Low => "{LOW}"
        }
    }
}

impl TodoManager {

    fn new(file_path: String) -> Self {
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(&parent);
        };
        let mut manager = TodoManager {
            file_path,
            current_todo_spaces: Vec::new(),
            current_space: None
        };
        if manager.load_todos().is_err() {
            manager.current_todo_spaces.push(TodoSpace { name: "Default".to_string(), todos: Vec::new() })
        }
        manager
    }

    fn add_todo(&mut self, todo_item: String, space_name: Option<String>, priority: Option<Priority>) -> Result<(), String> {
        let space_name = space_name.unwrap_or_else(|| "Default".to_string());
        let priority = priority.unwrap_or(Priority::Medium);

        // find or create the space     
        let space_name_idx = self.current_todo_spaces.iter()
            .position(|space| space.name == space_name)
            .unwrap_or_else(|| {
                // space doesnt exist, lets create it
                self.current_todo_spaces.push(
                    TodoSpace { name: space_name.clone(), todos: Vec::new() }
                );
                self.current_todo_spaces.len() - 1 //return the idx of the new space
            });
        self.current_todo_spaces[space_name_idx].todos.push(TodoItem{
            item: todo_item,
            status: false,
            priority,
            }); 
        self.save_todos()?;
        Ok(())

        
    }

    fn list_todos(&mut self) {
        for space in &self.current_todo_spaces {
            println!{"=== {} ===", space.name};
            for (i, todo) in space.todos.iter().enumerate() {
                let checkbox = if todo.status {"[x]"} else {"[ ]"};
                println!("{}: {} {} ({:?})", i, checkbox, todo.item, todo.priority);
            }
            println!();
        }
    }

    fn toggle_todo(&mut self, index: usize, space_name: Option<String>) -> Result<(), String> {
        let space_name = space_name.unwrap_or_else(|| "Default".to_string());
        if let Some(space) = self.current_todo_spaces.iter_mut().find(|s| s.name == space_name){
            if index >= space.todos.len() {
                return Err("Index out of bounds".to_string())
            };
            space.todos[index].status = !space.todos[index].status;
            self.save_todos()?;
            Ok(())
        } else {
            Err("Space not found".to_string())
        }
    }
    
    fn delete_todo(&mut self, index: usize, space_name: Option<String>) -> Result<(), String> {
        let space_name = space_name.unwrap_or_else(|| "Default".to_string());
        if let Some(space) = self.current_todo_spaces.iter_mut().find(|s| s.name == space_name) {
            if index >= space.todos.len() {
                return Err("Index out of bounds".to_string())
            };
            space.todos.remove(index);
            self.save_todos()?;
            Ok(())
        } else {
            Err("Space not found".to_string())
        }
    }

    fn save_todos(&self) -> Result<(), String> {
        let json_string = serde_json::to_string(&self.current_todo_spaces)
                .map_err(|e| e.to_string())?;

        fs::write(&self.file_path, json_string).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_todos(&mut self) -> Result<(), String> {
        let json_string = fs::read_to_string(&self.file_path)
            .map_err(|e| e.to_string())?;
        let loaded_spaces: Vec<TodoSpace> = serde_json::from_str(&json_string)
            .map_err(|e| e.to_string())?;
        self.current_todo_spaces = loaded_spaces;
        Ok(())
    }
}


// Helper fns

fn parse_markdown_todos(content: String) -> Vec<TodoSpace> {
    let mut spaces = Vec::new();
    let mut current_space = TodoSpace {
        name: "Default".to_string(),
        todos: Vec::new(),
    };

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            // Save current space if it has todos or is not Default
            if !current_space.todos.is_empty() || current_space.name != "Default" {
                spaces.push(current_space);
            }
            let space_name = trimmed.trim_start_matches("[[").trim_end_matches("]]");
            current_space = TodoSpace {
                name: space_name.to_string(),
                todos: Vec::new(),
            }
        } else if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let status = trimmed.starts_with("- [x]");
            let description = if status {
                trimmed.trim_start_matches("- [x] ")
            } else {
                trimmed.trim_start_matches("- [ ] ")
            };
            let priority = Priority::from_markdown(description);

            let item_text = description
                .replace("{LOW}", "")
                .replace("{MEDIUM}", "")
                .replace("{HIGH}", "")
                .replace("{URGENT}", "")
                .trim()
                .to_string();

            current_space.todos.push(
                TodoItem {
                    item: item_text,
                    status,
                    priority,
                }
            );
        }

    }
    if current_space.todos.is_empty() || current_space.name != "Default" {
        spaces.push(current_space)
    }
    spaces
}

fn format_todos_as_markdown(spaces: Vec<TodoSpace>) -> String {
    let mut markdown_string = String::new();
    for space in spaces.iter() {
        if space.name != "Default" {
            markdown_string.push_str(&format!("[[{}]]", space.name));
        }
        for todo in space.todos.iter() {
            let checkbox = if todo.status { "[x]" } else { "[ ]" };
            markdown_string.push_str(&format!("- {} {} {}\n", 
                checkbox, 
                todo.item, 
                todo.priority.to_markdown( ))
            )
        }
        markdown_string.push('\n');
    }
    markdown_string
}


#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A simple todo manager")]
struct Cli {
    #[arg(short, long)]
    add: Option<String>,

    #[arg(short, long)]
    toggle: Option<usize>,

    #[arg(short, long)]
    delete: Option<usize>,

    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<(), String> {
    let args = Cli::parse();
    let config_path = env::var("XDG_CONFIG_HOME").expect("$XDH_CONFIG_PATH not set");
    let path = config_path + "/todo" + "/todos.json";

    let mut manager = TodoManager::new(path.to_string());

    if let Some(todo_text) = args.add {
        manager.add_todo(todo_text, None, None)?;
        println!("Todo added!");
    }
    if let Some(idx) = args.toggle {
        manager.toggle_todo(idx, None)?;
        println!("Todo toggled!");
    }
    if let Some(idx) = args.delete {
        manager.delete_todo(idx, None)?;
    }
    if args.list {
        manager.list_todos();
    }
    Ok(())

}
