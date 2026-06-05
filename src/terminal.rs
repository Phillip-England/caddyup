use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::{App, Mode},
    caddyfile::CaddyDocument,
    tui,
};

pub fn run(document: CaddyDocument) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new(document);
    let result = run_loop(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| tui::draw(frame, app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(app, key);
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Navigate => handle_navigation_key(app, key),
        Mode::Edit => handle_edit_key(app, key),
    }
}

fn handle_navigation_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('s') => app.save(),
        KeyCode::Char('a') => app.add_site(),
        KeyCode::Char('d') => app.delete_site(),
        KeyCode::Up | KeyCode::Char('k') => app.move_field(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_field(1),
        KeyCode::Left | KeyCode::Char('h') => app.move_site(-1),
        KeyCode::Right | KeyCode::Char('l') => app.move_site(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.begin_edit(),
        _ => {}
    }
}

fn handle_edit_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Enter => app.commit_edit(),
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char(char) => app.input.push(char),
        _ => {}
    }
}
