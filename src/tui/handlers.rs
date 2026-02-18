use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, Focus};

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

        // Tab toggles focus between students list and content pane
        KeyCode::Tab => {
            app.toggle_focus();
            Action::None
        }

        // Left/Right change tabs
        KeyCode::Left | KeyCode::Char('h') => {
            app.prev_tab();
            Action::None
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.next_tab();
            Action::None
        }

        // Up/Down behavior depends on focus
        KeyCode::Down | KeyCode::Char('j') => {
            match app.focus {
                Focus::Students => app.next_student(),
                _ => app.scroll_down(),
            }
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.focus {
                Focus::Students => app.prev_student(),
                _ => app.scroll_up(),
            }
            Action::None
        }

        // Number keys for quick student selection (always works)
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

        // Refresh
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('R') => Action::RefreshAll,

        _ => Action::None,
    }
}
