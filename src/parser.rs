// TODO: Refactor this parser to use the rust-norg library once compatibility issues are resolved
// see https://github.com/nvim-neorg/rust-norg/pull/23
use crate::task::{Task, TodoItem, TodoState};
use color_eyre::Result;
use std::fs;
use std::path::Path;

pub struct NorgParser;

impl NorgParser {
    pub fn parse_task_file(file_path: &Path) -> Result<Task> {
        let content = fs::read_to_string(file_path)?;

        // For now, we'll skip the rust-norg parser since it has compatibility issues
        // and directly parse the content ourselves

        // Extract title from the first heading
        let title = Self::extract_title(&content);
        let mut task = Task::new(title, file_path.to_path_buf());

        // Extract description (content before first todo list)
        task.description = Self::extract_description(&content);

        // Parse todo items
        task.todos = Self::extract_todos(&content)?;

        Ok(task)
    }

    fn extract_title(content: &str) -> String {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('*') {
                // Remove leading asterisks and whitespace
                let title = trimmed.trim_start_matches('*').trim();
                if !title.is_empty() {
                    return title.to_string();
                }
            }
        }
        "Untitled Task".to_string()
    }

    fn extract_description(content: &str) -> String {
        let mut description_lines = Vec::new();
        let mut in_description = false;
        let mut found_title = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip title line
            if !found_title && trimmed.starts_with('*') {
                found_title = true;
                continue;
            }

            // Stop at first todo item
            if Self::is_todo_line(trimmed) {
                break;
            }

            if found_title {
                if trimmed.is_empty() && !in_description {
                    continue; // Skip empty lines before description
                }
                in_description = true;
                description_lines.push(line);
            }
        }

        description_lines.join("\n").trim().to_string()
    }

    fn extract_todos(content: &str) -> Result<Vec<TodoItem>> {
        let mut todos = Vec::new();
        let mut todo_id_counter = 0;

        for (line_number, line) in content.lines().enumerate() {
            if let Some(todo) = Self::parse_todo_line(line, line_number + 1)? {
                let mut todo = todo;
                todo.id = format!("todo_{}", todo_id_counter);
                todo_id_counter += 1;
                todos.push(todo);
            }
        }

        Ok(todos)
    }

    fn parse_todo_line(line: &str, line_number: usize) -> Result<Option<TodoItem>> {
        let trimmed = line.trim();

        if !Self::is_todo_line(trimmed) {
            return Ok(None);
        }

        // Count leading whitespace for level calculation
        let level = (line.len() - line.trim_start().len()) / 2; // Assuming 2 spaces per level

        // Find the todo marker pattern: ( ) or (x) etc.
        if let Some(marker_start) = trimmed.find('(') {
            if let Some(marker_end) = trimmed.find(')') {
                if marker_end > marker_start && marker_end - marker_start == 2 {
                    let state_char = trimmed.chars().nth(marker_start + 1).unwrap_or(' ');
                    let state = TodoState::from_norg_char(state_char).unwrap_or(TodoState::Undone);

                    // Extract text after the marker
                    let text = trimmed[marker_end + 1..].trim().to_string();

                    return Ok(Some(TodoItem {
                        id: String::new(), // Will be set by caller
                        text,
                        state,
                        level,
                        line_number,
                    }));
                }
            }
        }

        Ok(None)
    }

    fn is_todo_line(line: &str) -> bool {
        // Look for patterns like "- ( )", "- (x)" etc.
        (line.contains("- (") && line.contains(')'))
    }

    pub fn write_task_file(task: &Task) -> Result<()> {
        let mut content = String::new();

        // Write title
        content.push_str(&format!("* {}\n\n", task.title));

        // Write description
        if !task.description.is_empty() {
            content.push_str(&task.description);
            content.push_str("\n\n");
        }

        // Write todos
        for todo in &task.todos {
            let indent = "  ".repeat(todo.level);
            content.push_str(&format!(
                "{}- ({}) {}\n",
                indent,
                todo.state.to_norg_char(),
                todo.text
            ));
        }

        fs::write(&task.file_path, content)?;
        Ok(())
    }
}
