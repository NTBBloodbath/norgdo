use crate::app::{App, AppMode, FocusedPane};
use crate::task::{KanbanCategory, TodoState};
use ratatui::widgets::BorderType;
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap,
    },
};

pub fn render(app: &mut App, frame: &mut Frame) {
    match &app.mode.clone() {
        AppMode::Dashboard => render_dashboard(app, frame),
        AppMode::TaskDetail(task_id) => {
            let task_id = task_id.clone();
            render_task_detail(app, frame, &task_id);
        }
        AppMode::CreateTask => render_create_task(app, frame),
        AppMode::Search => render_search(app, frame),
        AppMode::Help => render_help(app, frame),
    }

    // Render error message if present
    if let Some(error) = &app.error_message {
        render_error_popup(frame, error);
    }
}

fn render_dashboard(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Help
        ])
        .split(frame.area());

    // Title
    let title = Paragraph::new("NorgDo - Terminal Task Manager")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(title, chunks[0]);

    // Main kanban board
    let kanban_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    // Get tasks and focused pane before rendering columns
    let tasks_by_category = app.task_manager.get_tasks_by_category();
    let focused_pane = app.focused_pane.clone();

    // Render kanban columns one at a time to avoid borrowing conflicts
    render_single_kanban_column(
        &mut app.list_states,
        frame,
        kanban_chunks[0],
        KanbanCategory::YetToBeDone,
        &tasks_by_category,
        focused_pane == FocusedPane::YetToBeDone,
    );

    render_single_kanban_column(
        &mut app.list_states,
        frame,
        kanban_chunks[1],
        KanbanCategory::InProgress,
        &tasks_by_category,
        focused_pane == FocusedPane::InProgress,
    );

    render_single_kanban_column(
        &mut app.list_states,
        frame,
        kanban_chunks[2],
        KanbanCategory::Completed,
        &tasks_by_category,
        focused_pane == FocusedPane::Completed,
    );

    // Help text
    let help_text = "Press ? for help | Press Esc/q to quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Help"),
        );
    frame.render_widget(help, chunks[2]);
}

fn render_single_kanban_column(
    list_states: &mut std::collections::HashMap<KanbanCategory, ratatui::widgets::ListState>,
    frame: &mut Frame,
    area: Rect,
    category: KanbanCategory,
    tasks_by_category: &std::collections::HashMap<KanbanCategory, Vec<&crate::task::Task>>,
    is_focused: bool,
) {
    let empty_vec = vec![];
    let tasks = tasks_by_category.get(&category).unwrap_or(&empty_vec);

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let completion = task.completion_percentage();
            let todo_counts = task.todo_counts();
            let total_todos = task.todos.len();

            let title_line = Line::from(vec![Span::styled(
                &task.title,
                Style::default().add_modifier(Modifier::BOLD),
            )]);

            let progress_line = if total_todos > 0 {
                // Create visual progress bar with block characters
                let bar_width = 20; // Total width of progress bar
                let filled_width = ((completion / 100.0) * bar_width as f64) as usize;
                let empty_width = bar_width - filled_width;

                let progress_bar =
                    format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width));

                Line::from(vec![Span::styled(
                    format!(
                        "{} {:.0}% ({}/{})",
                        progress_bar,
                        completion,
                        todo_counts.get(&TodoState::Done).unwrap_or(&0),
                        total_todos
                    ),
                    Style::default().fg(Color::Gray),
                )])
            } else {
                Line::from(vec![Span::styled(
                    "No todos",
                    Style::default().fg(Color::Gray),
                )])
            };

            ListItem::new(vec![title_line, progress_line]).style(Style::default().fg(Color::White))
        })
        .collect();

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(format!("{} ({})", category.to_string(), tasks.len())),
        )
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Never)
        .highlight_style(Style::default().bg(Color::Black))
        .highlight_symbol("» ");

    let state = list_states.get_mut(&category).unwrap();
    frame.render_stateful_widget(list, area, state);
}

fn render_task_detail(app: &mut App, frame: &mut Frame, task_id: &str) {
    if let Some(task) = app
        .task_manager
        .get_tasks()
        .iter()
        .find(|t| t.id == task_id)
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Description
                Constraint::Min(0),    // Todos
                Constraint::Length(3), // Help
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new(task.title.clone())
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Task"),
            );
        frame.render_widget(title, chunks[0]);

        // Description
        let description = if task.description.is_empty() {
            "No description provided.".to_string()
        } else {
            task.description.clone()
        };
        let desc_widget = Paragraph::new(description).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Description"),
        );
        frame.render_widget(desc_widget, chunks[1]);

        // Todos
        let todo_items: Vec<ListItem> = task
            .todos
            .iter()
            .map(|todo| {
                let indent = "  ".repeat(todo.level);
                let state_symbol = match todo.state {
                    TodoState::Done => "",
                    TodoState::Cancelled => "",
                    TodoState::Pending => "",
                    TodoState::Urgent => "",
                    TodoState::OnHold => "",
                    TodoState::Uncertain => "",
                    TodoState::Recurring => "",
                    TodoState::Undone => "",
                };

                let color = match todo.state {
                    TodoState::Done => Color::Green,
                    TodoState::Cancelled => Color::Red,
                    TodoState::Urgent => Color::Yellow,
                    TodoState::Pending => Color::Blue,
                    TodoState::Uncertain => Color::Magenta,
                    TodoState::OnHold => Color::Cyan,
                    TodoState::Recurring => Color::LightYellow,
                    TodoState::Undone => Color::White,
                };

                ListItem::new(Line::from(vec![
                    Span::raw(indent),
                    Span::styled(state_symbol, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(&todo.text, Style::default().fg(color)),
                    Span::styled(
                        format!(" ({})", todo.state.to_string()),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let todos_list = List::new(todo_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!("Todo Items ({})", task.todos.len())),
            )
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Never)
            .highlight_style(Style::default().bg(Color::Black))
            .highlight_symbol("» ");
        frame.render_stateful_widget(todos_list, chunks[2], &mut app.todo_list_state);

        // Help
        let help = Paragraph::new("Press ? for help | Press Esc/q to quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Help"),
            );
        frame.render_widget(help, chunks[3]);
    }
}

fn render_create_task(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let title = Paragraph::new("Create New Task")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(title, chunks[0]);

    let input = Paragraph::new(app.new_task_title.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Task Title"),
    );
    frame.render_widget(input, chunks[1]);

    let help = Paragraph::new("Type task title and press Enter to create, Esc to cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(help, chunks[2]);
}

fn render_search(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let title = Paragraph::new("Search Tasks")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(title, chunks[0]);

    let input = Paragraph::new(app.search_query.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Search Query"),
    );
    frame.render_widget(input, chunks[1]);

    // Show search results
    let search_results = app.task_manager.search_tasks(&app.search_query);
    let result_items: Vec<ListItem> = search_results
        .iter()
        .map(|task| {
            ListItem::new(Line::from(vec![
                Span::styled(&task.title, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" ({}% complete)", task.completion_percentage() as u8),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let results_list = List::new(result_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!("Results ({})", search_results.len())),
    );
    frame.render_widget(results_list, chunks[2]);
}

fn render_error_popup(frame: &mut Frame, error: &str) {
    let popup_area = centered_rect(60, 20, frame.area());

    frame.render_widget(Clear, popup_area);

    let error_widget = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
                .title("Error"),
        );

    frame.render_widget(error_widget, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help(app: &mut App, frame: &mut Frame) {
    let popup_area = centered_rect(80, 70, frame.area());

    frame.render_widget(Clear, popup_area);

    let help_content = vec![
        "DASHBOARD NAVIGATION:",
        "  Left/Right (← →)    Switch between kanban columns",
        "  Up/Down (↑ ↓)       Navigate within a column",
        "  Enter               Open selected task details",
        "",
        "TASK MANAGEMENT:",
        "  n                   Create new task",
        "  r                   Refresh tasks from disk",
        "  /                   Search tasks",
        "",
        "TASK DETAIL VIEW:",
        "  Up/Down (↑ ↓)       Navigate between TODO items",
        "  Space               Toggle TODO state (cycle through states)",
        "  s                   Save changes to file",
        "  Esc/q               Return to dashboard",
        "",
        "TODO STATES:",
        "   Undone            Task not started",
        "   Pending           Task in progress",
        "   Done              Task completed",
        "   Urgent            High priority task",
        "   Uncertain         Task status unclear",
        "   On Hold           Task paused",
        "   Cancelled         Task cancelled",
        "   Recurring         Recurring task",
        "",
        "GENERAL:",
        "  ?                   Show/hide this help",
        "  q                   Quit application",
        "",
        "HELP NAVIGATION:",
        "  Up/Down (↑ ↓)       Scroll help content",
        "  Page Up/Down        Scroll faster",
        "  Home                Go to top",
        "  End                 Go to bottom",
        "",
        "Press ? or Esc/q to close this help",
    ];

    // Calculate visible content based on scroll offset and available height
    let content_height = popup_area.height.saturating_sub(2) as usize; // Account for borders
    let scroll_offset = app.help_scroll_offset as usize;
    let max_scroll = help_content.len().saturating_sub(content_height);

    // Clamp scroll offset to valid range and update app state
    let actual_scroll = scroll_offset.min(max_scroll);
    app.help_scroll_offset = actual_scroll as u16;

    // Get visible lines
    let visible_content: Vec<&str> = help_content
        .iter()
        .skip(actual_scroll)
        .take(content_height)
        .cloned()
        .collect();

    let help_text = visible_content.join("\n");

    // Create the main content area and scrollbar area
    let content_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width.saturating_sub(1), // Leave space for scrollbar
        height: popup_area.height,
    };

    let scrollbar_area = Rect {
        x: popup_area.x + popup_area.width.saturating_sub(1),
        y: popup_area.y + 1, // Start below the top border
        width: 1,
        height: popup_area.height.saturating_sub(2), // Exclude top and bottom borders
    };

    // Render the help content
    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Help - Norgdo Terminal Task Manager"),
        );

    frame.render_widget(help_widget, content_area);

    // Update and render the scrollbar
    // FIXME: The damn scrollbar is not reaching the bottom when the help content reaches EOF
    // ratatui documentation is terrible so it might take a while for me to fix it
    app.help_scrollbar_state = app
        .help_scrollbar_state
        .content_length(help_content.len())
        .viewport_content_length(visible_content.len())
        .position(actual_scroll);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut app.help_scrollbar_state);
}
