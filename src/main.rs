mod app;
mod parser;
mod task;
mod task_manager;
mod ui;

use app::App;
use color_eyre::Result;
use ratatui::{DefaultTerminal, Frame};

fn render(app: &mut App, frame: &mut Frame) {
    ui::render(app, frame);
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new()?;

    loop {
        terminal.draw(|frame| render(&mut app, frame))?;

        app.handle_events()?;

        if app.should_quit {
            break Ok(());
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}
