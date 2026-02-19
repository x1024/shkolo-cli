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
    pub fn help_tabs(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[</>] Раздели", Lang::En => "[</>]Tabs" }
    }
    pub fn help_select(lang: Lang) -> &'static str {
        match lang { Lang::Bg => "[^v] Избор", Lang::En => "[^v]Select" }
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

}
