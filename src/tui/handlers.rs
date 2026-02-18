use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::i18n::Lang;
use super::app::{App, Focus, Tab};

pub enum Action {
    None,
    Refresh,
    RefreshAll,
    Logout,
    LoginPassword,
    LoginGoogle,
    ImportToken,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Handle Ctrl+C
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.quit();
        return Action::None;
    }

    // Settings tab has special key bindings
    if app.current_tab == Tab::Settings {
        match key.code {
            KeyCode::Char('g') | KeyCode::Char('G') => {
                // Toggle language
                app.lang = match app.lang {
                    Lang::Bg => Lang::En,
                    Lang::En => Lang::Bg,
                };
                return Action::None;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                return Action::Logout;
            }
            KeyCode::Char('1') => {
                return Action::LoginPassword;
            }
            KeyCode::Char('2') => {
                return Action::LoginGoogle;
            }
            KeyCode::Char('3') => {
                return Action::ImportToken;
            }
            _ => {}
        }
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

        // Number keys for quick student selection (only when not on Settings tab)
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

        // Resize students pane
        KeyCode::Char('[') => {
            app.resize_students_pane(-2);
            Action::None
        }
        KeyCode::Char(']') => {
            app.resize_students_pane(2);
            Action::None
        }

        _ => Action::None,
    }
}
