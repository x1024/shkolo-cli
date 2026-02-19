/// Simple internationalization module
/// Default language is Bulgarian (bg)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Bg,
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Bg  // Bulgarian is the default
    }
}

/// Translation strings
pub struct T;

impl T {
    // App title
    pub fn app_title(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Школо", Lang::En => "Shkolo" }
    }

    // Tab names
    pub fn overview(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Преглед", Lang::En => "Overview" }
    }
    pub fn homework(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Домашни", Lang::En => "Homework" }
    }
    pub fn grades(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Оценки", Lang::En => "Grades" }
    }
    pub fn schedule(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Програма", Lang::En => "Schedule" }
    }
    pub fn notifications(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Известия", Lang::En => "Notifications" }
    }
    pub fn absences(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отсъствия", Lang::En => "Absences" }
    }
    pub fn feedbacks(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отзиви", Lang::En => "Feedbacks" }
    }
    pub fn messages(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Съобщения", Lang::En => "Messages" }
    }
    pub fn settings(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Настройки", Lang::En => "Settings" }
    }

    // Feedbacks
    pub fn no_feedbacks(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма отзиви", Lang::En => "No feedbacks" }
    }
    pub fn positive(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "положителни", Lang::En => "positive" }
    }
    pub fn negative(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "отрицателни", Lang::En => "negative" }
    }

    // Messages
    pub fn no_messages(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма съобщения", Lang::En => "No messages" }
    }
    pub fn participants(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "участници", Lang::En => "participants" }
    }

    // Absences
    pub fn no_absences(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма отсъствия", Lang::En => "No absences" }
    }
    pub fn excused(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "извинено", Lang::En => "excused" }
    }
    pub fn unexcused(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "неизвинено", Lang::En => "unexcused" }
    }
    pub fn hour_label(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "час", Lang::En => "hour" }
    }

    // Section titles
    pub fn students(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Ученици", Lang::En => "Students" }
    }
    pub fn today_schedule(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Днешна програма", Lang::En => "Today's Schedule" }
    }
    pub fn recent_homework(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Скорошни домашни", Lang::En => "Recent Homework" }
    }
    pub fn grades_summary(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Обобщение оценки", Lang::En => "Grades Summary" }
    }
    pub fn total_grades(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Общо оценки", Lang::En => "Total grades" }
    }

    // Status messages
    pub fn loading(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Зареждане...", Lang::En => "Loading..." }
    }
    pub fn loading_base(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Зареждане", Lang::En => "Loading" }
    }
    pub fn no_homework(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма домашни", Lang::En => "No homework found" }
    }
    pub fn no_grades(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма оценки", Lang::En => "No grades found" }
    }
    pub fn no_schedule(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма часове за днес", Lang::En => "No classes scheduled" }
    }
    pub fn no_notifications(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Няма известия", Lang::En => "No notifications" }
    }
    pub fn no_student(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Не е избран ученик", Lang::En => "No student selected" }
    }

    // Labels
    pub fn past_due(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Минал", Lang::En => "Past" }
    }
    pub fn term1(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Срок 1", Lang::En => "Term 1" }
    }
    pub fn term2(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Срок 2", Lang::En => "Term 2" }
    }
    pub fn final_grade(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Срочна", Lang::En => "Final" }
    }
    pub fn annual(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Годишна", Lang::En => "Annual" }
    }
    pub fn teacher(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Учител", Lang::En => "Teacher" }
    }
    pub fn topic(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Тема", Lang::En => "Topic" }
    }
    pub fn unread(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "непрочетени", Lang::En => "unread" }
    }
    pub fn new_marker(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[НОВО] ", Lang::En => "[NEW] " }
    }

    // Help text
    pub fn help_refresh(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[R] Обнови", Lang::En => "[R]efresh" }
    }
    pub fn help_quit(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[Q] Изход", Lang::En => "[Q]uit" }
    }
    pub fn help_help(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[?] Помощ", Lang::En => "[?]Help" }
    }

    // Status bar
    pub fn last_refresh(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Последно:", Lang::En => "Last:" }
    }
    pub fn loading_data(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Зареждане данни...", Lang::En => "Loading data..." }
    }
    pub fn loading_thread(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Зареждане нишка...", Lang::En => "Loading thread..." }
    }
    pub fn loading_recipients(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Зареждане получатели...", Lang::En => "Loading recipients..." }
    }
    pub fn error_prefix(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Грешка:", Lang::En => "Error:" }
    }
    pub fn failed_load_thread(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Грешка при зареждане на нишка:", Lang::En => "Failed to load thread:" }
    }
    pub fn loaded(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Заредено", Lang::En => "Loaded" }
    }
    pub fn sending(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изпращане...", Lang::En => "Sending..." }
    }
    pub fn sending_message(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изпращане на съобщение...", Lang::En => "Sending message..." }
    }
    pub fn message_sent(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Съобщението е изпратено!", Lang::En => "Message sent!" }
    }
    pub fn sent_reload_failed(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изпратено, но презареждане се провали:", Lang::En => "Sent, but reload failed:" }
    }
    pub fn send_failed(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Грешка при изпращане:", Lang::En => "Send failed:" }
    }
    pub fn logout_error(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Грешка при изход:", Lang::En => "Logout error:" }
    }
    pub fn logged_out(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Излязохте. Рестартирайте за нов вход.", Lang::En => "Logged out. Restart to log in again." }
    }

    // Settings/Account
    pub fn account(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Акаунт", Lang::En => "Account" }
    }
    pub fn logged_in_as(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Вписан като", Lang::En => "Logged in as" }
    }
    pub fn logout(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изход от акаунт", Lang::En => "Logout" }
    }
    pub fn login(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Вход", Lang::En => "Login" }
    }
    pub fn login_password(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Вход с парола", Lang::En => "Login with password" }
    }
    pub fn login_google(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Вход с Google", Lang::En => "Login with Google" }
    }
    pub fn import_token(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Импорт на токен", Lang::En => "Import token" }
    }
    pub fn import_token_desc(lang: Lang) -> &'static str {
        match lang {
            Lang::Bg => "Импортира токен от iOS приложението Shkolo на този Mac",
            Lang::En => "Import token from the Shkolo iOS app on this Mac"
        }
    }

    // Keybinding descriptions
    pub fn key_show_help(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Покажи/скрий помощ", Lang::En => "Show/hide help" }
    }
    pub fn key_quit(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изход", Lang::En => "Quit" }
    }
    pub fn key_cancel_input(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отказ", Lang::En => "Cancel input" }
    }
    pub fn key_submit(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изпрати/напред", Lang::En => "Submit/next field" }
    }
    pub fn key_delete_char(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Изтрий символ", Lang::En => "Delete character" }
    }
    pub fn key_move_cursor(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Курсор ляво/дясно", Lang::En => "Move cursor" }
    }
    pub fn key_jump_start_end(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "В начало/края", Lang::En => "Jump to start/end" }
    }
    pub fn key_move_to_body(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Към текст", Lang::En => "Move to message body" }
    }
    pub fn key_close_thread(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Затвори", Lang::En => "Close thread" }
    }
    pub fn key_reply(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отговор", Lang::En => "Reply to thread" }
    }
    pub fn key_scroll(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Превърти", Lang::En => "Scroll" }
    }
    pub fn key_cancel_compose(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отказ", Lang::En => "Cancel compose" }
    }
    pub fn key_navigate(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Навигация", Lang::En => "Navigate" }
    }
    pub fn key_toggle_recipient(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Избери/премахни", Lang::En => "Toggle recipient" }
    }
    pub fn key_start_subject(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Напиши тема", Lang::En => "Start writing subject" }
    }
    pub fn key_switch_tabs(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Смени раздел", Lang::En => "Switch tabs" }
    }
    pub fn key_toggle_focus(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Смени фокус", Lang::En => "Toggle focus (students/content)" }
    }
    pub fn key_navigate_scroll(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Навигация / Превърти", Lang::En => "Navigate / Scroll" }
    }
    pub fn key_quick_select_tab(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Бърз избор раздел", Lang::En => "Quick select tab" }
    }
    pub fn key_refresh(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Обнови", Lang::En => "Refresh data" }
    }
    pub fn key_force_refresh(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Обнови всичко", Lang::En => "Force refresh all" }
    }
    pub fn key_toggle_lang(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Смени език (БГ/EN)", Lang::En => "Toggle language (BG/EN)" }
    }
    pub fn key_resize_pane(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Преоразмери панел", Lang::En => "Resize students pane" }
    }
    pub fn key_resize_split(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Преоразмери разделител", Lang::En => "Resize split" }
    }
    pub fn key_prev_day(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Предишен ден", Lang::En => "Previous day" }
    }
    pub fn key_next_day(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Следващ ден", Lang::En => "Next day" }
    }
    pub fn key_go_today(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Днес", Lang::En => "Go to today" }
    }
    pub fn key_go_to_tab(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Към свързан раздел", Lang::En => "Go to related tab" }
    }
    pub fn key_open_thread(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отвори", Lang::En => "Open thread" }
    }
    pub fn key_compose(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Ново съобщение", Lang::En => "Compose new message" }
    }
    pub fn keyboard_shortcuts(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Клавишни комбинации", Lang::En => "Keyboard Shortcuts" }
    }
    pub fn press_any_key(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Натисни клавиш", Lang::En => "Press any key" }
    }

    // Context descriptions for help overlay
    pub fn ctx_replying(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Отговор", Lang::En => "Replying" }
    }
    pub fn ctx_composing_subject(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Тема", Lang::En => "Composing Subject" }
    }
    pub fn ctx_composing_body(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Съобщение", Lang::En => "Composing Message" }
    }
    pub fn ctx_thread_view(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Преглед нишка", Lang::En => "Thread View" }
    }
    pub fn ctx_select_recipients(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "Избор получатели", Lang::En => "Select Recipients" }
    }

}
