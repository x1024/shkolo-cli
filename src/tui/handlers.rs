use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::App;

pub enum Action {
    None,
    Refresh,
    RefreshAll,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Handle Ctrl+C
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.quit();
        return Action::None;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.quit();
            Action::None
        }

        // Tab navigation
        KeyCode::Tab => {
            app.next_tab();
            Action::None
        }
        KeyCode::BackTab => {
            app.prev_tab();
            Action::None
        }

        // List navigation
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_down();
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_up();
            Action::None
        }

        // Student selection
        KeyCode::Char('1') => {
            app.select_student(0);
            Action::None
        }
        KeyCode::Char('2') => {
            app.select_student(1);
            Action::None
        }
        KeyCode::Char('3') => {
            app.select_student(2);
            Action::None
        }
        KeyCode::Char('4') => {
            app.select_student(3);
            Action::None
        }
        KeyCode::Char('5') => {
            app.select_student(4);
            Action::None
        }

        // Next/prev student
        KeyCode::Right | KeyCode::Char('l') => {
            app.next_student();
            Action::None
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.prev_student();
            Action::None
        }

        // Refresh
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('R') => Action::RefreshAll,

        _ => Action::None,
    }
}
