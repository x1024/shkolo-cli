use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::i18n::{Lang, T};
use super::app::{App, Focus, Tab, InputMode, MessageView};

pub enum Action {
    None,
    Refresh,
    RefreshAll,
    RefreshSchedule, // Refresh schedule for current schedule_date
    Logout,
    // Message actions
    OpenThread(i64),       // Open thread with given ID
    CloseThread,           // Close current thread
    SendReply(String),     // Send reply message
    StartCompose,          // Start composing a new message
    SendCompose { subject: String, body: String, recipients: Vec<i64> }, // Send new message
    // Navigation history
    NavigateBack,          // Go back in history (may need to reload data)
    NavigateForward,       // Go forward in history (may need to reload data)
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Handle Ctrl+C (always works)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.quit();
        return Action::None;
    }

    // Handle ? for help (always works, toggles help overlay)
    if key.code == KeyCode::Char('?') {
        app.toggle_help();
        return Action::None;
    }

    // Any key dismisses help overlay if shown
    if app.show_help {
        app.show_help = false;
        return Action::None;
    }

    // Dismiss error on any key
    if app.error_message.is_some() {
        app.clear_error();
        return Action::None;
    }

    // Handle input mode first (for reply/compose)
    if app.input_mode != InputMode::Normal {
        return handle_input_mode(app, key);
    }

    // Handle message thread view
    if app.current_tab == Tab::Messages && app.message_view == MessageView::Thread {
        return handle_thread_view(app, key);
    }

    // Handle compose view
    if app.current_tab == Tab::Messages && app.message_view == MessageView::Compose {
        return handle_compose_view(app, key);
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
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Cycle auto-refresh interval
                app.next_auto_refresh();
                return Action::None;
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
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('[') => {
            app.prev_tab();
            Action::None
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(']') => {
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

        // Number keys for quick tab selection (1-9)
        // Note: On Settings tab, 1-3 are handled above for login options
        KeyCode::Char('1') => { app.select_tab(0); Action::None }
        KeyCode::Char('2') => { app.select_tab(1); Action::None }
        KeyCode::Char('3') => { app.select_tab(2); Action::None }
        KeyCode::Char('4') => { app.select_tab(3); Action::None }
        KeyCode::Char('5') => { app.select_tab(4); Action::None }
        KeyCode::Char('6') => { app.select_tab(5); Action::None }
        KeyCode::Char('7') => { app.select_tab(6); Action::None }
        KeyCode::Char('8') => { app.select_tab(7); Action::None }
        KeyCode::Char('9') => { app.select_tab(8); Action::None }

        // Refresh
        KeyCode::Char('r') => {
            // On Schedule tab, refresh the selected date's schedule
            if app.current_tab == Tab::Schedule {
                Action::RefreshSchedule
            } else {
                Action::Refresh
            }
        }
        KeyCode::Char('R') => Action::RefreshAll,

        // Resize students pane (horizontal)
        KeyCode::Char('-') => {
            app.resize_students_pane(-2);
            app.set_status(format!("Pane width: {}", app.students_pane_width));
            Action::None
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.resize_students_pane(2);
            app.set_status(format!("Pane width: {}", app.students_pane_width));
            Action::None
        }

        // Resize overview split (vertical) - only on Overview tab
        KeyCode::Char('<') => {
            if app.current_tab == Tab::Overview {
                app.resize_overview_split(-5);
                app.set_status(format!("Overview split: {}%", app.overview_split_percent));
            }
            Action::None
        }
        KeyCode::Char('>') => {
            if app.current_tab == Tab::Overview {
                app.resize_overview_split(5);
                app.set_status(format!("Overview split: {}%", app.overview_split_percent));
            }
            Action::None
        }

        // Enter to activate/select
        KeyCode::Enter => {
            // On Notifications tab, navigate to related tab
            if app.current_tab == Tab::Notifications {
                app.activate_notification();
            }
            // On Messages tab, open the selected thread
            else if app.current_tab == Tab::Messages {
                if let Some(thread_id) = app.open_thread() {
                    return Action::OpenThread(thread_id);
                }
            }
            Action::None
        }

        // 'c' to compose new message (only on Messages tab)
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if app.current_tab == Tab::Messages && app.message_view == MessageView::List {
                app.start_compose();
                return Action::StartCompose;
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

        // Navigation history: Backspace = back, Shift+Backspace or Alt+Right = forward
        KeyCode::Backspace => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Forward
                if app.go_forward() {
                    // Check if we need to reload thread messages
                    if app.message_view == MessageView::Thread {
                        if let Some(thread_id) = app.selected_thread_id {
                            return Action::OpenThread(thread_id);
                        }
                    }
                }
            } else {
                // Back
                if app.go_back() {
                    // Check if we need to reload thread messages
                    if app.message_view == MessageView::Thread {
                        if let Some(thread_id) = app.selected_thread_id {
                            return Action::OpenThread(thread_id);
                        }
                    }
                }
            }
            Action::None
        }

        _ => Action::None,
    }
}

/// Handle keys when in input mode (reply/compose)
fn handle_input_mode(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // Escape cancels input
        KeyCode::Esc => {
            match app.input_mode {
                InputMode::Reply => {
                    app.cancel_input();
                }
                InputMode::ComposeSubject | InputMode::ComposeBody => {
                    app.cancel_compose();
                }
                _ => {}
            }
            Action::None
        }
        // Tab cycles forward, Shift+Tab cycles back in compose mode
        KeyCode::Tab | KeyCode::BackTab => {
            let is_back = key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT);
            if is_back {
                app.compose_prev_step();
            } else {
                match app.input_mode {
                    InputMode::ComposeSubject => app.compose_next_step(),
                    InputMode::ComposeBody => {
                        // Tab in body cycles back to recipients
                        app.compose_prev_step(); // body -> subject
                        app.compose_prev_step(); // subject -> recipients
                    }
                    _ => {}
                }
            }
            Action::None
        }
        // Enter submits the input
        KeyCode::Enter => {
            match app.input_mode {
                InputMode::Reply => {
                    if !app.input_buffer.is_empty() {
                        let message = app.take_input();
                        return Action::SendReply(message);
                    }
                }
                InputMode::ComposeSubject => {
                    // Move to body entry
                    app.compose_next_step();
                }
                InputMode::ComposeBody => {
                    // Send the composed message
                    if app.can_send_compose() {
                        let subject = app.compose_subject.clone();
                        let body = app.input_buffer.clone();
                        let recipients = app.selected_recipients.clone();
                        app.cancel_compose();
                        return Action::SendCompose { subject, body, recipients };
                    }
                }
                _ => {}
            }
            Action::None
        }
        // Backspace deletes character before cursor
        KeyCode::Backspace => {
            app.input_backspace();
            Action::None
        }
        // Delete deletes character at cursor
        KeyCode::Delete => {
            app.input_delete();
            Action::None
        }
        // Left/Right move cursor
        KeyCode::Left => {
            app.input_left();
            Action::None
        }
        KeyCode::Right => {
            app.input_right();
            Action::None
        }
        // Home/End for cursor
        KeyCode::Home => {
            app.input_cursor = 0;
            Action::None
        }
        KeyCode::End => {
            app.input_cursor = app.input_buffer.len();
            Action::None
        }
        // Character input
        KeyCode::Char(c) => {
            app.input_char(c);
            Action::None
        }
        _ => Action::None,
    }
}

/// Handle keys when in compose view (recipient selection)
fn handle_compose_view(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // Escape cancels compose
        KeyCode::Esc => {
            app.cancel_compose();
            Action::None
        }
        // Enter or Space toggles recipient selection
        KeyCode::Enter | KeyCode::Char(' ') => {
            if app.input_mode == InputMode::Normal {
                app.toggle_recipient(app.list_offset);
            }
            Action::None
        }
        // 's' to start entering subject
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if !app.selected_recipients.is_empty() {
                app.input_mode = InputMode::ComposeSubject;
            }
            Action::None
        }
        // Tab moves to subject (regardless of selection), Shift+Tab cycles from recipients to body
        KeyCode::Tab => {
            app.input_mode = InputMode::ComposeSubject;
            Action::None
        }
        KeyCode::BackTab => {
            // Shift+Tab from recipients cycles to body
            app.input_mode = InputMode::ComposeBody;
            Action::None
        }
        // Up/Down to navigate recipients
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.recipients.len().saturating_sub(1);
            if app.list_offset < max {
                app.list_offset += 1;
            }
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.list_offset = app.list_offset.saturating_sub(1);
            Action::None
        }
        _ => Action::None,
    }
}

/// Handle keys when viewing a message thread
fn handle_thread_view(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // Escape, q, or Backspace closes the thread view (goes back)
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
            // Use navigation history to go back
            if app.go_back() {
                // Successfully went back - thread is already closed via apply_location
                Action::CloseThread
            } else {
                // Fallback if no history (shouldn't happen normally)
                app.close_thread();
                Action::CloseThread
            }
        }
        // r starts reply mode
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.start_reply();
            Action::None
        }
        // j/k or Down/Up scroll messages
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.thread_messages.len().saturating_sub(1);
            if app.thread_offset < max {
                app.thread_offset += 1;
            }
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.thread_offset = app.thread_offset.saturating_sub(1);
            Action::None
        }
        _ => Action::None,
    }
}

/// Get context-aware keybindings for the current app state
/// Returns a list of (key, description) pairs
/// This is defined here alongside the actual key handlers to keep them in sync
pub fn get_keybindings(app: &App) -> Vec<(&'static str, &'static str)> {
    let lang = app.lang;
    let mut bindings = Vec::new();

    // Always available
    bindings.push(("?", T::key_show_help(lang)));

    // Check for special modes first
    if app.input_mode != InputMode::Normal {
        // Input mode keybindings (see handle_input_mode)
        bindings.push(("Esc", T::key_cancel_input(lang)));
        bindings.push(("Enter", T::key_submit(lang)));
        bindings.push(("Backspace", T::key_delete_char(lang)));
        bindings.push(("←/→", T::key_move_cursor(lang)));
        bindings.push(("Home/End", T::key_jump_start_end(lang)));
        if app.input_mode == InputMode::ComposeSubject {
            bindings.push(("Tab", T::key_move_to_body(lang)));
        }
        return bindings;
    }

    // Message thread view (see handle_thread_view)
    if app.current_tab == Tab::Messages && app.message_view == MessageView::Thread {
        bindings.push(("⌫/Esc/q", T::key_go_back(lang)));
        bindings.push(("r", T::key_reply(lang)));
        bindings.push(("↓/j ↑/k", T::key_scroll(lang)));
        return bindings;
    }

    // Compose view - recipient selection (see handle_compose_view)
    if app.current_tab == Tab::Messages && app.message_view == MessageView::Compose {
        bindings.push(("Esc", T::key_cancel_compose(lang)));
        bindings.push(("↓/j ↑/k", T::key_navigate(lang)));
        bindings.push(("Enter/Space", T::key_toggle_recipient(lang)));
        bindings.push(("s", T::key_start_subject(lang)));
        return bindings;
    }

    // Normal mode - common bindings (see handle_key)
    // q/Esc/Ctrl+C all quit - consolidated into one entry
    bindings.push(("q/Esc/^C", T::key_quit(lang)));
    bindings.push(("←/h/[ →/l/]", T::key_switch_tabs(lang)));
    bindings.push(("Tab", T::key_toggle_focus(lang)));
    bindings.push(("↓/j ↑/k", T::key_navigate_scroll(lang)));
    bindings.push(("1-9", T::key_quick_select_tab(lang)));
    bindings.push(("r", T::key_refresh(lang)));
    bindings.push(("R", T::key_force_refresh(lang)));
    bindings.push(("G", T::key_toggle_lang(lang)));
    bindings.push(("-/+/=", T::key_resize_pane(lang)));
    bindings.push(("⌫", T::key_go_back(lang)));
    bindings.push(("⇧⌫", T::key_go_forward(lang)));

    // Tab-specific bindings
    match app.current_tab {
        Tab::Overview => {
            bindings.push(("</>", T::key_resize_split(lang)));
        }
        Tab::Schedule => {
            bindings.push(("p", T::key_prev_day(lang)));
            bindings.push(("n", T::key_next_day(lang)));
            bindings.push(("t", T::key_go_today(lang)));
        }
        Tab::Notifications => {
            bindings.push(("Enter", T::key_go_to_tab(lang)));
        }
        Tab::Messages => {
            bindings.push(("Enter", T::key_open_thread(lang)));
            bindings.push(("c", T::key_compose(lang)));
        }
        Tab::Settings => {
            bindings.push(("L", T::logout(lang)));
        }
        _ => {}
    }

    bindings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_refresh_on_schedule_tab_refreshes_selected_date() {
        let mut app = App::new();

        // On Overview tab, 'r' should return Refresh
        app.current_tab = Tab::Overview;
        let action = handle_key(&mut app, key_event(KeyCode::Char('r')));
        assert!(matches!(action, Action::Refresh));

        // On Schedule tab, 'r' should return RefreshSchedule
        app.current_tab = Tab::Schedule;
        let action = handle_key(&mut app, key_event(KeyCode::Char('r')));
        assert!(matches!(action, Action::RefreshSchedule));

        // On Homework tab, 'r' should return Refresh
        app.current_tab = Tab::Homework;
        let action = handle_key(&mut app, key_event(KeyCode::Char('r')));
        assert!(matches!(action, Action::Refresh));
    }

    #[test]
    fn test_refresh_all_works_on_any_tab() {
        let mut app = App::new();

        // 'R' should always return RefreshAll regardless of tab
        app.current_tab = Tab::Overview;
        let action = handle_key(&mut app, key_event(KeyCode::Char('R')));
        assert!(matches!(action, Action::RefreshAll));

        app.current_tab = Tab::Schedule;
        let action = handle_key(&mut app, key_event(KeyCode::Char('R')));
        assert!(matches!(action, Action::RefreshAll));
    }

    #[test]
    fn test_auto_refresh_toggle_on_settings() {
        use crate::tui::app::AutoRefreshInterval;

        let mut app = App::new();
        app.current_tab = Tab::Settings;

        // Default is 10 minutes
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min10);

        // Press 'a' to cycle to next interval (30 min)
        let action = handle_key(&mut app, key_event(KeyCode::Char('a')));
        assert!(matches!(action, Action::None));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min30);

        // Continue cycling: 30 -> 60 -> Off -> 1 -> 5 -> 10
        handle_key(&mut app, key_event(KeyCode::Char('A')));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min60);

        handle_key(&mut app, key_event(KeyCode::Char('a')));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Off);

        handle_key(&mut app, key_event(KeyCode::Char('a')));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min1);

        handle_key(&mut app, key_event(KeyCode::Char('a')));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min5);

        handle_key(&mut app, key_event(KeyCode::Char('a')));
        assert_eq!(app.auto_refresh_interval, AutoRefreshInterval::Min10);
    }

    #[test]
    fn test_auto_refresh_interval_minutes() {
        use crate::tui::app::AutoRefreshInterval;

        assert_eq!(AutoRefreshInterval::Off.minutes(), None);
        assert_eq!(AutoRefreshInterval::Min1.minutes(), Some(1));
        assert_eq!(AutoRefreshInterval::Min5.minutes(), Some(5));
        assert_eq!(AutoRefreshInterval::Min10.minutes(), Some(10));
        assert_eq!(AutoRefreshInterval::Min30.minutes(), Some(30));
        assert_eq!(AutoRefreshInterval::Min60.minutes(), Some(60));
    }
}
