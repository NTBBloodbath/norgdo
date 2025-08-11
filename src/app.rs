use crate::task::KanbanCategory;
use crate::task_manager::TaskManager;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Dashboard,
    TaskDetail(String), // task_id
    CreateTask,
    CreateTaskWizard(WizardStep),
    Search,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Title,
    Description,
    Todos,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct TaskWizardData {
    pub title: String,
    pub description: String,
    pub todos: Vec<String>,
    pub current_todo: String,
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
    pub todo_list_state: ListState, // For navigating todos in task detail view
    pub help_scroll_offset: u16,    // For scrolling help content
    pub help_scrollbar_state: ScrollbarState, // For help scrollbar widget
    pub wizard_data: TaskWizardData, // For task creation wizard
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
            todo_list_state: ListState::default(),
            help_scroll_offset: 0,
            help_scrollbar_state: ScrollbarState::default(),
            wizard_data: TaskWizardData {
                title: String::new(),
                description: String::new(),
                todos: Vec::new(),
                current_todo: String::new(),
            },
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
                }
                AppMode::CreateTask => self.handle_create_task_input(key.code)?,
                AppMode::CreateTaskWizard(step) => {
                    let step = step.clone();
                    self.handle_wizard_input(key.code, step)?;
                }
                AppMode::Search => self.handle_search_input(key.code)?,
                AppMode::Help => self.handle_help_input(key.code)?,
            }
        }
        Ok(())
    }

    fn handle_dashboard_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('n') => {
                // Reset wizard data and start the wizard
                self.wizard_data = TaskWizardData {
                    title: String::new(),
                    description: String::new(),
                    todos: Vec::new(),
                    current_todo: String::new(),
                };
                self.mode = AppMode::CreateTaskWizard(WizardStep::Title);
            }
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_query.clear();
            }
            KeyCode::Left => match self.focused_pane {
                FocusedPane::YetToBeDone => {}
                FocusedPane::InProgress => self.focused_pane = FocusedPane::YetToBeDone,
                FocusedPane::Completed => self.focused_pane = FocusedPane::InProgress,
            },
            KeyCode::Right => match self.focused_pane {
                FocusedPane::YetToBeDone => self.focused_pane = FocusedPane::InProgress,
                FocusedPane::InProgress => self.focused_pane = FocusedPane::Completed,
                FocusedPane::Completed => {}
            },
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Enter => self.open_selected_task()?,
            KeyCode::Char('r') => {
                self.task_manager.load_tasks()?;
            }
            KeyCode::Char('?') => {
                self.mode = AppMode::Help;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_task_detail_input(&mut self, key_code: KeyCode, task_id: &str) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Char('q') => self.mode = AppMode::Dashboard,
            KeyCode::Char('s') => {
                self.task_manager.save_task(task_id)?;
            }
            KeyCode::Up => {
                // Navigate up in todo list
                if let Some(task) = self
                    .task_manager
                    .get_tasks()
                    .iter()
                    .find(|t| t.id == task_id)
                {
                    let todo_count = task.todos.len();
                    if todo_count > 0 {
                        let current = self.todo_list_state.selected().unwrap_or(0);
                        let new_index = current.saturating_sub(1);
                        self.todo_list_state.select(Some(new_index));
                    }
                }
            }
            KeyCode::Down => {
                // Navigate down in todo list
                if let Some(task) = self
                    .task_manager
                    .get_tasks()
                    .iter()
                    .find(|t| t.id == task_id)
                {
                    let todo_count = task.todos.len();
                    if todo_count > 0 {
                        let current = self.todo_list_state.selected().unwrap_or(0);
                        let new_index = (current + 1).min(todo_count - 1);
                        self.todo_list_state.select(Some(new_index));
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Toggle todo state
                if let Some(selected_index) = self.todo_list_state.selected() {
                    self.task_manager
                        .toggle_todo_state(task_id, selected_index)?;
                }
            }
            KeyCode::Char('?') => {
                self.mode = AppMode::Help;
            }
            _ => {}
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
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to create task: {}", e));
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                self.new_task_title.pop();
            }
            KeyCode::Char(c) => {
                self.new_task_title.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                // TODO: Implement search results view
                self.mode = AppMode::Dashboard;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.mode = AppMode::Dashboard;
                self.help_scroll_offset = 0; // Reset scroll when closing help
                self.help_scrollbar_state = ScrollbarState::default(); // Reset scrollbar state
            }
            KeyCode::Up => {
                if self.help_scroll_offset > 0 {
                    self.help_scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                // We'll clamp this in the UI render function based on content size
                self.help_scroll_offset += 1;
            }
            KeyCode::PageUp => {
                self.help_scroll_offset = self.help_scroll_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.help_scroll_offset += 5;
            }
            KeyCode::Home => {
                self.help_scroll_offset = 0;
            }
            KeyCode::End => {
                self.help_scroll_offset = u16::MAX; // Set to max to scroll to bottom
            }
            _ => {}
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
        let tasks_in_category = tasks_by_category
            .get(&current_category)
            .map(|v| v.len())
            .unwrap_or(0);

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
                        // Reset todo list state when entering task detail
                        self.todo_list_state.select(Some(0));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_wizard_input(&mut self, key_code: KeyCode, step: WizardStep) -> Result<()> {
        match step {
            WizardStep::Title => self.handle_wizard_title_input(key_code)?,
            WizardStep::Description => self.handle_wizard_description_input(key_code)?,
            WizardStep::Todos => self.handle_wizard_todos_input(key_code)?,
            WizardStep::Confirm => self.handle_wizard_confirm_input(key_code)?,
        }
        Ok(())
    }

    fn handle_wizard_title_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                if !self.wizard_data.title.trim().is_empty() {
                    self.mode = AppMode::CreateTaskWizard(WizardStep::Description);
                }
            }
            KeyCode::Backspace => {
                self.wizard_data.title.pop();
            }
            KeyCode::Char(c) => {
                self.wizard_data.title.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_wizard_description_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                // Move to todos step regardless of description content
                self.mode = AppMode::CreateTaskWizard(WizardStep::Todos);
            }
            KeyCode::Backspace => {
                self.wizard_data.description.pop();
            }
            KeyCode::Char(c) => {
                self.wizard_data.description.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_wizard_todos_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter => {
                if !self.wizard_data.current_todo.trim().is_empty() {
                    // Add current todo to the list
                    self.wizard_data
                        .todos
                        .push(self.wizard_data.current_todo.clone());
                    self.wizard_data.current_todo.clear();
                } else {
                    // If current todo is empty, move to confirm step
                    self.mode = AppMode::CreateTaskWizard(WizardStep::Confirm);
                }
            }
            KeyCode::Backspace => {
                self.wizard_data.current_todo.pop();
            }
            KeyCode::Char(c) => {
                self.wizard_data.current_todo.push(c);
            }
            KeyCode::Tab => {
                // Skip to confirm step
                if !self.wizard_data.current_todo.trim().is_empty() {
                    self.wizard_data
                        .todos
                        .push(self.wizard_data.current_todo.clone());
                    self.wizard_data.current_todo.clear();
                }
                self.mode = AppMode::CreateTaskWizard(WizardStep::Confirm);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_wizard_confirm_input(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Create the task
                match self.task_manager.create_task_with_details(
                    self.wizard_data.title.clone(),
                    self.wizard_data.description.clone(),
                    self.wizard_data.todos.clone(),
                ) {
                    Ok(_) => {
                        self.mode = AppMode::Dashboard;
                        // Reset wizard data
                        self.wizard_data = TaskWizardData {
                            title: String::new(),
                            description: String::new(),
                            todos: Vec::new(),
                            current_todo: String::new(),
                        };
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create task: {}", e));
                        self.mode = AppMode::Dashboard;
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.mode = AppMode::Dashboard;
            }
            _ => {}
        }
        Ok(())
    }
}
