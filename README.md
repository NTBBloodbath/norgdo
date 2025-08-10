# Norgdo

Get your tasks done using Norg, straight from your terminal.

> Norgdo is a terminal-based task manager for the Norg markup format, built with Rust and ratatui.

> [!IMPORTANT]
>
> This software is in alpha status. While it works, it is uncomplete and might have bugs. Please report bugs back to me if you spot any.

## Features

- **Kanban-style interface** with three columns: Yet to be Done, In Progress, and Completed
- **Norg file format support** for task management with proper TODO states
- **Real-time progress tracking** with completion percentages
- **Terminal-based UI** using ratatui for a responsive interface
- **Task categorization** based on TODO states automatically
- **Search functionality** to find tasks quickly
- **Create new tasks** directly from the terminal interface

## Installation

1. Clone the repository:
```bash
git clone https://github.com/NTBBloodbath/norgdo
cd norgdo
```

2. Build with Cargo:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run
```

## Usage

### Basic Navigation

- **←→** Switch between kanban columns (Yet to be Done, In Progress, Completed)
- **↑↓** Navigate within a column to select tasks
- **Enter** Open selected task for detailed view
- **n** Create a new task
- **/** Search for tasks
- **r** Refresh task list from disk
- **q** Quit the application

### Enhanced Navigation (Task Detail View)

- **↑↓** Navigate between TODO items within a task
- **Space** Toggle TODO state (cycles through Undone → Pending → Done)
- **s** Save changes to the task file
- **Esc/q** Return to main dashboard

### Task File Format

Norgdo uses the Norg markup format for task files. Tasks are stored as `.norg` files in `~/.local/share/norgdo/` (or `$XDG_DATA_HOME/norgdo`).

#### Norg TODO States

Norgdo supports all Norg TODO states as mentioned in the Neovim Neorg plugin:

- `( )` - **Undone** - Task not started
- `(x)` - **Done** - Task completed
- `(-)` - **Pending** - Task in progress
- `(!)` - **Urgent** - High priority task
- `(?)` - **Uncertain** - Task status unclear
- `(=)` - **On Hold** - Task paused
- `(_)` - **Cancelled** - Task cancelled
- `(+)` - **Recurring** - Recurring task

#### Sample Task File

```norg
* Project Setup

This is a sample project to demonstrate norgdo functionality.

- ( ) Set up development environment
- (x) Create project structure
- (-) Write documentation
- (!) Fix critical bug
- ( ) Write comprehensive tests
-- ( ) Unit tests
-- ( ) Integration tests
- (_) Remove deprecated features
```

### Kanban Categories

Tasks are automatically categorized based on their TODO states:

- **Yet to be Done**: Tasks with only undone, uncertain, or on-hold todos
- **In Progress**: Tasks with pending, urgent, or mixed completed/uncompleted todos
- **Completed**: Tasks where all todos are done or cancelled

### Creating Tasks

1. Press `n` to create a new task
2. Type the task title
3. Press `Enter` to create the task file
4. The file will be saved in your data directory as `<title>.norg`
5. Edit the file manually to add description and TODO items

### Task Detail View

1. Select a task and press `Enter` to view details
2. See task title, description, and all TODO items with their states
3. **Navigate TODOs**: Use `Up/Down` arrows to select specific TODO items
4. **Toggle TODO states**: Press `Space` to cycle through states (Undone → Pending → Done → Undone)
5. **Save changes**: Press `s` to save TODO state changes to the file
6. View completion progress and todo counts
7. Press `Esc` or `q` to return to the main dashboard

## File Structure

```
~/.local/share/norgdo/           # Data directory
├── project_setup.norg           # Individual task files
├── learning_rust.norg
└── documentation.norg
```

## Roadmap

- [x] Basic kanban interface
- [x] Task file parsing
- [x] Navigation and task selection
- [x] Task editing interface
- [x] TODO state toggling
- [x] Visual progress bars
- [x] Enhanced TODO navigation in detail view
- [ ] Task relationships (dependencies, related tasks)
- [ ] Due date support
- [ ] Search and filtering
- [ ] Task creation wizard
- [ ] Export functionality
- [ ] Configuration options

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the GPL-2.0 License - see the [LICENSE](./LICENSE) file for details.
