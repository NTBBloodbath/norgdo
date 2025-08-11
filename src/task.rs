use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TodoState {
    Done,
    Pending,
    Undone,
    Uncertain,
    OnHold,
    Cancelled,
    Recurring,
    Urgent,
}

impl TodoState {
    pub fn from_norg_char(c: char) -> Option<Self> {
        match c {
            'x' => Some(TodoState::Done),
            '-' => Some(TodoState::Pending),
            ' ' => Some(TodoState::Undone),
            '?' => Some(TodoState::Uncertain),
            '=' => Some(TodoState::OnHold),
            '_' => Some(TodoState::Cancelled),
            '+' => Some(TodoState::Recurring),
            '!' => Some(TodoState::Urgent),
            _ => None,
        }
    }

    pub fn to_norg_char(&self) -> char {
        match self {
            TodoState::Done => 'x',
            TodoState::Pending => '-',
            TodoState::Undone => ' ',
            TodoState::Uncertain => '?',
            TodoState::OnHold => '=',
            TodoState::Cancelled => '_',
            TodoState::Recurring => '+',
            TodoState::Urgent => '!',
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            TodoState::Done => "Done",
            TodoState::Pending => "Pending",
            TodoState::Undone => "To Do",
            TodoState::Uncertain => "Uncertain",
            TodoState::OnHold => "On Hold",
            TodoState::Cancelled => "Cancelled",
            TodoState::Recurring => "Recurring",
            TodoState::Urgent => "Urgent",
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, TodoState::Done | TodoState::Cancelled)
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self, TodoState::Pending | TodoState::Urgent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub state: TodoState,
    pub level: usize, // Indentation level for sub-todos
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRelation {
    pub target_task_id: String,
    pub relation_type: RelationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    Related,
    Requires,
    Blocks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub todos: Vec<TodoItem>,
    pub relations: Vec<TaskRelation>,
    pub file_path: PathBuf,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    pub fn new(title: String, file_path: PathBuf) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description: String::new(),
            todos: Vec::new(),
            relations: Vec::new(),
            file_path,
            due_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn todo_counts(&self) -> HashMap<TodoState, usize> {
        let mut counts = HashMap::new();
        for todo in &self.todos {
            *counts.entry(todo.state.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn completion_percentage(&self) -> f64 {
        if self.todos.is_empty() {
            return 100.0;
        }

        let completed = self
            .todos
            .iter()
            .filter(|todo| todo.state.is_completed())
            .count();

        (completed as f64 / self.todos.len() as f64) * 100.0
    }

    pub fn kanban_category(&self) -> KanbanCategory {
        if self.todos.is_empty() {
            return KanbanCategory::YetToBeDone;
        }

        let counts = self.todo_counts();
        let total = self.todos.len();
        let completed = *counts.get(&TodoState::Done).unwrap_or(&0)
            + *counts.get(&TodoState::Cancelled).unwrap_or(&0);
        let in_progress = *counts.get(&TodoState::Pending).unwrap_or(&0)
            + *counts.get(&TodoState::Urgent).unwrap_or(&0);

        if completed == total {
            KanbanCategory::Completed
        } else if in_progress > 0 || completed > 0 {
            KanbanCategory::InProgress
        } else {
            KanbanCategory::YetToBeDone
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KanbanCategory {
    YetToBeDone,
    InProgress,
    Completed,
}

impl KanbanCategory {
    pub fn to_string(&self) -> &'static str {
        match self {
            KanbanCategory::YetToBeDone => "Yet to be Done",
            KanbanCategory::InProgress => "In Progress",
            KanbanCategory::Completed => "Completed",
        }
    }
}
