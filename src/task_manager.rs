use crate::parser::NorgParser;
use crate::task::{KanbanCategory, Task};
use color_eyre::Result;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct TaskManager {
    tasks: Vec<Task>,
    data_dir: PathBuf,
}

impl TaskManager {
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_directory()?;

        // Ensure data directory exists
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let mut manager = Self {
            tasks: Vec::new(),
            data_dir,
        };

        manager.load_tasks()?;
        Ok(manager)
    }

    fn get_data_directory() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "norgdo") {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            // Fallback to ~/.local/share/norgdo
            let home = dirs::home_dir()
                .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine home directory"))?;
            Ok(home.join(".local").join("share").join("norgdo"))
        }
    }

    pub fn load_tasks(&mut self) -> Result<()> {
        self.tasks.clear();

        if !self.data_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "norg") {
                match NorgParser::parse_task_file(&path) {
                    Ok(task) => self.tasks.push(task),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse task file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn create_task(&mut self, title: String) -> Result<&Task> {
        let filename = Self::sanitize_filename(&title);
        let file_path = self.data_dir.join(format!("{}.norg", filename));

        let task = Task::new(title, file_path);
        NorgParser::write_task_file(&task)?;

        self.tasks.push(task);
        Ok(self.tasks.last().unwrap())
    }

    pub fn save_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            NorgParser::write_task_file(task)?;
        }
        Ok(())
    }

    pub fn delete_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(index) = self.tasks.iter().position(|t| t.id == task_id) {
            let task = &self.tasks[index];
            if task.file_path.exists() {
                fs::remove_file(&task.file_path)?;
            }
            self.tasks.remove(index);
        }
        Ok(())
    }

    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    pub fn get_tasks_by_category(&self) -> HashMap<KanbanCategory, Vec<&Task>> {
        let mut categorized = HashMap::new();

        for task in &self.tasks {
            let category = task.kanban_category();
            categorized
                .entry(category)
                .or_insert_with(Vec::new)
                .push(task);
        }

        categorized
    }

    pub fn search_tasks(&self, query: &str) -> Vec<&Task> {
        let query_lower = query.to_lowercase();
        self.tasks
            .iter()
            .filter(|task| {
                task.title.to_lowercase().contains(&query_lower)
                    || task.description.to_lowercase().contains(&query_lower)
                    || task
                        .todos
                        .iter()
                        .any(|todo| todo.text.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn toggle_todo_state(&mut self, task_id: &str, todo_index: usize) -> Result<()> {
        use crate::task::TodoState;

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if let Some(todo) = task.todos.get_mut(todo_index) {
                // Toggle between common states: Undone -> Pending -> Done -> Undone
                todo.state = match todo.state {
                    TodoState::Undone => TodoState::Pending,
                    TodoState::Pending => TodoState::Done,
                    TodoState::Done => TodoState::Undone,
                    TodoState::Urgent => TodoState::Done,
                    TodoState::Uncertain => TodoState::Pending,
                    TodoState::OnHold => TodoState::Pending,
                    TodoState::Cancelled => TodoState::Undone,
                    TodoState::Recurring => TodoState::Done,
                };

                // Save the task file with updated TODO states
                self.save_task(task_id)?;
            }
        }
        Ok(())
    }

    fn sanitize_filename(title: &str) -> String {
        title
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                ' ' => '_',
                _ => '_',
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }
}
