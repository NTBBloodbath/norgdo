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

## Showcase

<details>
   <summary>Kanban UI</summary>

   <img width="1884" height="1000" alt="image" src="https://github.com/user-attachments/assets/47108205-0a15-4baf-b911-04716eac9afb" />
</details>

<details>
   <summary>Task UI</summary>

   <img width="1884" height="1000" alt="image" src="https://github.com/user-attachments/assets/3e10a2cb-470c-4847-a05a-31a5ba1963d6" />
</details>

<details>
   <summary>TODO state change UI</summary>

   <img width="1884" height="1000" alt="image" src="https://github.com/user-attachments/assets/e64ca773-d9e7-4363-aad3-ce87534d9959" />
</details>

<details>
   <summary>Help UI</summary>
   
   <img width="1884" height="1000" alt="image" src="https://github.com/user-attachments/assets/f2d8a6b4-d42d-4dd3-8b8b-78ec810e905e" />
</details>

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

> [!IMPORTANT]
>
> Norgdo requires nerd fonts to display the TODO icons!

### Basic Navigation

- **←→** Switch between kanban columns (Yet to be Done, In Progress, Completed)
- **↑↓** Navigate within a column to select tasks
- **Enter** Open selected task for detailed view
- **n** Create a new task
- **/** Search for tasks
- **r** Refresh task list from disk
- **?** Show help popup
- **q** Quit the application

### Task Detail View Navigation

- **↑↓** Navigate between TODO items within a task
- **Space** Open TODO state selection dialog (choose from all 8 states)
- **s** Save changes to the task file
- **Esc/q** Return to main dashboard

### TODO State Selection

Press `Space` on any TODO item to open an interactive state selection dialog:
- **↑↓** Navigate through all available states
- **Enter/Space** Apply the selected state
- **Esc/q** Cancel selection

Choose from any of the 8 Norg TODO states with visual indicators and descriptions.

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

Norgdo features a **4-step task creation wizard** with full navigation and editing capabilities:

1. **Press `n`** to start the task creation wizard
2. **Step 1 - Title**: Type the task title and press `Enter`
3. **Step 2 - Description**: Type an optional description and press `Enter`
4. **Step 3 - TODO Items**:
   - Type TODO items one by one, pressing `Enter` after each
   - Use `↑↓` arrows to navigate between existing TODO items
   - Press `Delete` to remove selected TODO items
   - Press `F2` to edit selected TODO items
   - Press `Enter` on an empty line or `Tab` to skip to confirmation
5. **Step 4 - Confirmation**: Review your task and press `Y` to create or `N` to cancel

The wizard creates complete `.norg` files with proper formatting in your data directory (`~/.local/share/norgdo/`).

#### Wizard Keybinds
- **Enter**: Continue to next step / Add TODO item
- **Tab**: Skip to confirmation (from TODO step)
- **← (Left Arrow)**: Go back to previous step
- **Backspace**: Delete character / Go back to previous step (when input is empty)
- **↑↓ (Up/Down)**: Navigate TODO list for editing (step 3 only)
- **Delete**: Remove selected TODO item (step 3 only)
- **F2**: Edit selected TODO item (step 3 only)
- **Y/N**: Confirm or cancel task creation (final step)
- **Tab**: Skip to confirmation (from TODO step)
- **Esc**: Cancel wizard and return to dashboard
- **Backspace**: Delete characters while typing
- **Y/N**: Confirm or cancel task creation (final step)

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
- [x] Task creation wizard
- [ ] Task relationships (dependencies, related tasks)
- [ ] Due date support
- [ ] Search and filtering
- [ ] Export functionality
- [ ] Configuration options

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the GPL-2.0 License - see the [LICENSE](./LICENSE) file for details.
