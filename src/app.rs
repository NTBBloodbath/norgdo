use crate::task::KanbanCategory;
use crate::task_manager::TaskManager;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    widgets::{ListState},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Dashboard,
    TaskDetail(String), // task_id
    CreateTask,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPane {
    YetToBeDone,
    InProgress,
    Completed,
}

pub struct App {
    pub task_manager: TaskManager,
    pub mode: AppMode,
    pub focused_pane: FocusedPane,
    pub list_states: HashMap<KanbanCategory, ListState>,
    pub should_quit: bool,
    pub search_query: String,
    pub new_task_title: String,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let task_manager = TaskManager::new()?;
        let mut list_states = HashMap::new();

        list_states.insert(KanbanCategory::YetToBeDone, ListState::default());
        list_states.insert(KanbanCategory::InProgress, ListState::default());
        list_states.insert(KanbanCategory::Completed, ListState::default());

        Ok(Self {
            task_manager,
            mode: AppMode::Dashboard,
            focused_pane: FocusedPane::YetToBeDone,
            list_states,
            should_quit: false,
            search_query: String::new(),
            new_task_title: String::new(),
            error_message: None,
        })
    }

    pub fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            // Clear error message on any key press
            self.error_message = None;

            match &self.mode {
                AppMode::Dashboard => self.handle_dashboard_input(key.code)?,
                AppMode::TaskDetail(task_id) => {
                    let task_id = task_id.clone();
                    self.handle_task_detail_input(key.code, &task_id)?;
                },
                AppMode::CreateTask => self.handle_create_task_input(key.code)?,
                AppMode::Search => self.handle_search_input(key.code)?,
            }
        }
        Ok(())
    }

    fn handle_dashboard_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('n') => {
                self.mode = AppMode::CreateTask;
                self.new_task_title.clear();
            },
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_query.clear();
            },
            KeyCode::Left => self.focused_pane = FocusedPane::YetToBeDone,
            KeyCode::Right => match self.focused_pane {
                FocusedPane::YetToBeDone => self.focused_pane = FocusedPane::InProgress,
                FocusedPane::InProgress => self.focused_pane = FocusedPane::Completed,
                FocusedPane::Completed => {},
            },
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Enter => self.open_selected_task()?,
            KeyCode::Char('r') => {
                self.task_manager.load_tasks()?;
            },
            _ => {},
        }
        Ok(())
    }

    fn handle_task_detail_input(&mut self, key_code: KeyCode, task_id: &str) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Char('q') => self.mode = AppMode::Dashboard,
            KeyCode::Char('s') => {
                self.task_manager.save_task(task_id)?;
            },
            // TODO: Add more task detail navigation and editing
            _ => {},
        }
        Ok(())
    }

    fn handle_create_task_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                if !self.new_task_title.trim().is_empty() {
                    match self.task_manager.create_task(self.new_task_title.clone()) {
                        Ok(_) => {
                            self.mode = AppMode::Dashboard;
                            self.new_task_title.clear();
                        },
                        Err(e) => {
                            self.error_message = Some(format!("Failed to create task: {}", e));
                        }
                    }
                }
            },
            KeyCode::Backspace => {
                self.new_task_title.pop();
            },
            KeyCode::Char(c) => {
                self.new_task_title.push(c);
            },
            _ => {},
        }
        Ok(())
    }

    fn handle_search_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                // TODO: Implement search results view
                self.mode = AppMode::Dashboard;
            },
            KeyCode::Backspace => {
                self.search_query.pop();
            },
            KeyCode::Char(c) => {
                self.search_query.push(c);
            },
            _ => {},
        }
        Ok(())
    }

    fn move_selection(&mut self, direction: i32) {
        let current_category = match self.focused_pane {
            FocusedPane::YetToBeDone => KanbanCategory::YetToBeDone,
            FocusedPane::InProgress => KanbanCategory::InProgress,
            FocusedPane::Completed => KanbanCategory::Completed,
        };

        let tasks_by_category = self.task_manager.get_tasks_by_category();
        let tasks_in_category = tasks_by_category.get(&current_category).map(|v| v.len()).unwrap_or(0);

        if tasks_in_category == 0 {
            return;
        }

        let state = self.list_states.get_mut(&current_category).unwrap();
        let current = state.selected().unwrap_or(0);

        let new_index = if direction > 0 {
            (current + 1).min(tasks_in_category - 1)
        } else {
            current.saturating_sub(1)
        };

        state.select(Some(new_index));
    }

    fn open_selected_task(&mut self) -> Result<()> {
        let current_category = match self.focused_pane {
            FocusedPane::YetToBeDone => KanbanCategory::YetToBeDone,
            FocusedPane::InProgress => KanbanCategory::InProgress,
            FocusedPane::Completed => KanbanCategory::Completed,
        };

        let tasks_by_category = self.task_manager.get_tasks_by_category();
        if let Some(tasks) = tasks_by_category.get(&current_category) {
            if let Some(state) = self.list_states.get(&current_category) {
                if let Some(selected) = state.selected() {
                    if let Some(task) = tasks.get(selected) {
                        self.mode = AppMode::TaskDetail(task.id.clone());
                    }
                }
            }
        }
        Ok(())
    }
}
