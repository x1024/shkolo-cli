use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::i18n::Lang;
use super::app::{App, Focus, Tab};

pub enum Action {
    None,
    Refresh,
    RefreshAll,
    RefreshSchedule, // Refresh schedule for current schedule_date
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
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('{') => {
            app.prev_tab();
            Action::None
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('}') => {
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
        KeyCode::Char('[') | KeyCode::Char('-') => {
            app.resize_students_pane(-2);
            app.set_status(format!("Pane width: {}", app.students_pane_width));
            Action::None
        }
        KeyCode::Char(']') | KeyCode::Char('+') | KeyCode::Char('=') => {
            app.resize_students_pane(2);
            app.set_status(format!("Pane width: {}", app.students_pane_width));
            Action::None
        }

        // Enter to activate/select
        KeyCode::Enter => {
            // On Notifications tab, navigate to related tab
            if app.current_tab == Tab::Notifications {
                app.activate_notification();
            }
            Action::None
        }

        // Schedule date navigation (only on Schedule tab)
        KeyCode::Char('n') => {
            if app.current_tab == Tab::Schedule {
                app.schedule_next_day();
                return Action::RefreshSchedule;
            }
            Action::None
        }
        KeyCode::Char('p') => {
            if app.current_tab == Tab::Schedule {
                app.schedule_prev_day();
                return Action::RefreshSchedule;
            }
            Action::None
        }
        KeyCode::Char('t') => {
            if app.current_tab == Tab::Schedule {
                app.schedule_today();
                return Action::RefreshSchedule;
            }
            Action::None
        }

        _ => Action::None,
    }
}
