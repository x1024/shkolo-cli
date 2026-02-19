use crate::api::ShkoloClient;
use crate::cache::CacheStore;
use crate::i18n::{Lang, T};
use crate::models::*;
use time::OffsetDateTime;

/// Input mode for text entry (reply/compose)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Reply,           // Replying to a thread
    ComposeSubject,  // Composing - entering subject
    ComposeBody,     // Composing - entering body
}

/// Message view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageView {
    List,    // Viewing message list
    Thread,  // Viewing a specific thread
    Compose, // Composing a new message
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Homework,
    Grades,
    Schedule,
    Absences,
    Feedbacks,
    Messages,
    Notifications,
    Settings,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Homework, Tab::Grades, Tab::Schedule, Tab::Absences, Tab::Feedbacks, Tab::Messages, Tab::Notifications, Tab::Settings]
    }

    pub fn name(&self, lang: Lang) -> &'static str {
        match self {
            Tab::Overview => T::overview(lang),
            Tab::Homework => T::homework(lang),
            Tab::Grades => T::grades(lang),
            Tab::Schedule => T::schedule(lang),
            Tab::Absences => T::absences(lang),
            Tab::Feedbacks => T::feedbacks(lang),
            Tab::Messages => T::messages(lang),
            Tab::Notifications => T::notifications(lang),
            Tab::Settings => T::settings(lang),
        }
    }

    pub fn next(&self) -> Tab {
        match self {
            Tab::Overview => Tab::Homework,
            Tab::Homework => Tab::Grades,
            Tab::Grades => Tab::Schedule,
            Tab::Schedule => Tab::Absences,
            Tab::Absences => Tab::Feedbacks,
            Tab::Feedbacks => Tab::Messages,
            Tab::Messages => Tab::Notifications,
            Tab::Notifications => Tab::Settings,
            Tab::Settings => Tab::Overview,
        }
    }

    pub fn prev(&self) -> Tab {
        match self {
            Tab::Overview => Tab::Settings,
            Tab::Homework => Tab::Overview,
            Tab::Grades => Tab::Homework,
            Tab::Schedule => Tab::Grades,
            Tab::Absences => Tab::Schedule,
            Tab::Feedbacks => Tab::Absences,
            Tab::Messages => Tab::Feedbacks,
            Tab::Notifications => Tab::Messages,
            Tab::Settings => Tab::Notifications,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Students,
    Content,
    // Overview tab sub-panes
    OverviewSchedule,
    OverviewHomework,
    OverviewGrades,
}

#[derive(Debug, Clone)]
pub struct StudentData {
    pub student: Student,
    pub homework: Vec<Homework>,
    pub grades: Vec<Grade>,
    pub schedule: Vec<ScheduleHour>,
    pub events: Vec<Event>,
    pub absences: Vec<Absence>,
    pub feedbacks: Vec<Feedback>,
    pub homework_age: Option<String>,
    pub grades_age: Option<String>,
    pub schedule_age: Option<String>,
    pub absences_age: Option<String>,
    pub feedbacks_age: Option<String>,
}

impl StudentData {
    pub fn new(student: Student) -> Self {
        Self {
            student,
            homework: Vec::new(),
            grades: Vec::new(),
            schedule: Vec::new(),
            events: Vec::new(),
            absences: Vec::new(),
            feedbacks: Vec::new(),
            homework_age: None,
            grades_age: None,
            schedule_age: None,
            absences_age: None,
            feedbacks_age: None,
        }
    }

    /// Count total grades across all subjects
    pub fn total_grades_count(&self) -> usize {
        self.grades.iter()
            .map(|g| g.term1_grades.len() + g.term2_grades.len())
            .sum()
    }

    /// Get recent grades (last few per subject)
    pub fn recent_grades_summary(&self) -> Vec<(&str, Vec<&str>)> {
        self.grades.iter()
            .filter(|g| !g.term1_grades.is_empty() || !g.term2_grades.is_empty())
            .take(5)
            .map(|g| {
                let recent: Vec<&str> = g.term2_grades.iter()
                    .chain(g.term1_grades.iter())
                    .rev()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect();
                (g.subject.as_str(), recent)
            })
            .collect()
    }
}

pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub focus: Focus,
    pub lang: Lang,
    pub user_name: Option<String>,
    pub students: Vec<StudentData>,
    pub selected_student: usize,
    pub list_offset: usize,
    // Separate scroll offsets for overview sub-panes
    pub schedule_offset: usize,
    pub homework_offset: usize,
    pub grades_offset: usize,
    pub notifications: Vec<Notification>,
    pub notifications_age: Option<String>,
    pub messages: Vec<MessageThread>,
    pub messages_age: Option<String>,
    pub status_message: Option<String>,
    pub error_message: Option<String>,  // Persistent error message
    pub loading: bool,
    pub last_refresh: Option<String>,
    pub current_date: String,
    pub schedule_date: String, // Date being viewed in schedule (can differ from current_date)
    pub current_time: (u8, u8), // (hour, minute)
    pub tick: usize, // Frame counter for animations
    pub students_pane_width: u16, // Resizable pane width
    pub overview_split_percent: u16, // Vertical split for overview (schedule vs homework/grades)
    // Message thread state
    pub message_view: MessageView,
    pub selected_thread_id: Option<i64>,
    pub thread_messages: Vec<Message>,
    pub thread_offset: usize,
    // Input mode for text entry
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
    // Recipients for composing
    pub recipients: Vec<Recipient>,
    pub selected_recipients: Vec<i64>,
    pub compose_subject: String,
    // Help overlay
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        // Use local time for schedule/homework comparison
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let today = format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day());
        Self {
            running: true,
            current_tab: Tab::Overview,
            focus: Focus::Students,
            lang: Lang::default(), // Bulgarian by default
            user_name: None,
            students: Vec::new(),
            selected_student: 0,
            list_offset: 0,
            schedule_offset: 0,
            homework_offset: 0,
            grades_offset: 0,
            notifications: Vec::new(),
            notifications_age: None,
            messages: Vec::new(),
            messages_age: None,
            status_message: None,
            error_message: None,
            loading: false,
            last_refresh: None,
            current_date: today.clone(),
            schedule_date: today,
            current_time: (now.hour(), now.minute()),
            tick: 0,
            students_pane_width: 30,
            overview_split_percent: 40, // 40% for schedule, 60% for homework/grades
            // Message thread state
            message_view: MessageView::List,
            selected_thread_id: None,
            thread_messages: Vec::new(),
            thread_offset: 0,
            // Input mode
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            // Compose state
            recipients: Vec::new(),
            selected_recipients: Vec::new(),
            compose_subject: String::new(),
            // Help
            show_help: false,
        }
    }

    /// Move schedule to next day
    pub fn schedule_next_day(&mut self) {
        if let Ok(date) = time::Date::parse(&self.schedule_date, time::macros::format_description!("[year]-[month]-[day]")) {
            let next = date + time::Duration::days(1);
            self.schedule_date = format!("{:04}-{:02}-{:02}", next.year(), next.month() as u8, next.day());
        }
    }

    /// Move schedule to previous day
    pub fn schedule_prev_day(&mut self) {
        if let Ok(date) = time::Date::parse(&self.schedule_date, time::macros::format_description!("[year]-[month]-[day]")) {
            let prev = date - time::Duration::days(1);
            self.schedule_date = format!("{:04}-{:02}-{:02}", prev.year(), prev.month() as u8, prev.day());
        }
    }

    /// Reset schedule to today
    pub fn schedule_today(&mut self) {
        self.schedule_date = self.current_date.clone();
    }

    /// Check if schedule is showing today
    pub fn is_schedule_today(&self) -> bool {
        self.schedule_date == self.current_date
    }

    pub fn resize_students_pane(&mut self, delta: i16) {
        let new_width = (self.students_pane_width as i16 + delta).clamp(15, 60) as u16;
        self.students_pane_width = new_width;
    }

    pub fn resize_overview_split(&mut self, delta: i16) {
        let new_percent = (self.overview_split_percent as i16 + delta).clamp(20, 70) as u16;
        self.overview_split_percent = new_percent;
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn update_time(&mut self) {
        // Use local time for schedule comparison (not UTC)
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        self.current_time = (now.hour(), now.minute());
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
        self.list_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.prev();
        self.list_offset = 0;
    }

    /// Set the current tab directly (for mouse click)
    pub fn set_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
        self.list_offset = 0;
    }

    /// Select tab by index (0-8 for 9 tabs)
    pub fn select_tab(&mut self, index: usize) {
        if let Some(&tab) = Tab::all().get(index) {
            self.set_tab(tab);
        }
    }

    /// Handle click on tab bar - returns true if a tab was clicked
    pub fn click_tab(&mut self, column: u16) -> bool {
        // Tab bar layout: " TabName " with borders
        // Approximate tab widths based on names (EN/BG)
        let tabs = Tab::all();
        let mut x = 1; // Start after left border

        for tab in tabs {
            let tab_width = tab.name(self.lang).chars().count() as u16 + 2; // +2 for padding

            if column >= x && column < x + tab_width {
                self.set_tab(*tab);
                return true;
            }

            x += tab_width + 1; // +1 for separator
        }

        false
    }

    /// Handle click on list item - selects and activates item based on row position
    /// Returns ClickResult indicating what action should be taken
    pub fn click_list_item(&mut self, row: u16, header_offset: u16, column: u16, students_width: u16) -> ClickResult {
        // row is absolute, we need to convert to list index
        // header_offset is the number of rows taken by the header (tab bar + borders)
        // Each pane also has its own border (1 row at top)
        let pane_border = 1u16;

        if row < header_offset + pane_border {
            return ClickResult::None;
        }

        let relative_row = (row - header_offset - pane_border) as usize;

        // Check if click is in students pane (left side)
        if column < students_width {
            // Clicking on a student selects them
            if relative_row < self.students.len() {
                self.selected_student = relative_row;
                self.list_offset = 0;
                return ClickResult::StudentSelected;
            }
            return ClickResult::None;
        }

        // Click is in content pane
        // Calculate the actual item index: scroll offset + row position in visible area
        let item_index = self.list_offset + relative_row;

        // Check bounds - clicking should NOT scroll, just select/activate the item
        if item_index < self.current_list_length() {
            // Return activation result based on current tab with the item index
            return match self.current_tab {
                Tab::Notifications => ClickResult::ActivateNotification(item_index),
                Tab::Messages => {
                    if self.message_view == MessageView::List {
                        ClickResult::ActivateMessage(item_index)
                    } else {
                        ClickResult::None
                    }
                }
                _ => ClickResult::ItemSelected(item_index),
            };
        }

        ClickResult::None
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.current_tab {
            Tab::Overview => {
                // Cycle: Students -> Schedule -> Homework -> Grades -> Students
                match self.focus {
                    Focus::Students => Focus::OverviewSchedule,
                    Focus::OverviewSchedule => Focus::OverviewHomework,
                    Focus::OverviewHomework => Focus::OverviewGrades,
                    Focus::OverviewGrades => Focus::Students,
                    _ => Focus::Students,
                }
            }
            _ => {
                // Other tabs: Students -> Content -> Students
                match self.focus {
                    Focus::Students => Focus::Content,
                    _ => Focus::Students,
                }
            }
        };
        self.list_offset = 0;
    }

    pub fn next_student(&mut self) {
        if !self.students.is_empty() {
            self.selected_student = (self.selected_student + 1) % self.students.len();
            self.list_offset = 0;
        }
    }

    pub fn prev_student(&mut self) {
        if !self.students.is_empty() {
            self.selected_student = if self.selected_student == 0 {
                self.students.len() - 1
            } else {
                self.selected_student - 1
            };
            self.list_offset = 0;
        }
    }

    pub fn select_student(&mut self, index: usize) {
        if index < self.students.len() {
            self.selected_student = index;
            self.list_offset = 0;
        }
    }

    /// Get the number of items in the current list (for scroll bounds)
    pub fn current_list_length(&self) -> usize {
        match self.current_tab {
            Tab::Notifications => self.notifications.len(),
            Tab::Messages => self.messages.len(),
            Tab::Homework => self.current_student().map(|s| s.homework.len()).unwrap_or(0),
            Tab::Grades => self.current_student().map(|s| s.grades.len()).unwrap_or(0),
            Tab::Schedule => self.current_student().map(|s| s.schedule.len()).unwrap_or(0),
            Tab::Absences => self.current_student().map(|s| s.absences.len()).unwrap_or(0),
            Tab::Feedbacks => self.current_student().map(|s| s.feedbacks.len()).unwrap_or(0),
            Tab::Overview | Tab::Settings => 0,
        }
    }

    /// Get the number of items in the current overview sub-pane (for scroll bounds)
    fn overview_list_length(&self) -> usize {
        match self.focus {
            Focus::OverviewSchedule => self.current_student().map(|s| s.schedule.len()).unwrap_or(0),
            Focus::OverviewHomework => self.current_student().map(|s| s.homework.len().min(5)).unwrap_or(0),
            Focus::OverviewGrades => self.current_student().map(|s| s.grades.len().min(5)).unwrap_or(0),
            _ => 0,
        }
    }

    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::OverviewSchedule => {
                let max = self.overview_list_length().saturating_sub(1);
                if self.schedule_offset < max {
                    self.schedule_offset = self.schedule_offset.saturating_add(1);
                }
            }
            Focus::OverviewHomework => {
                let max = self.overview_list_length().saturating_sub(1);
                if self.homework_offset < max {
                    self.homework_offset = self.homework_offset.saturating_add(1);
                }
            }
            Focus::OverviewGrades => {
                let max = self.overview_list_length().saturating_sub(1);
                if self.grades_offset < max {
                    self.grades_offset = self.grades_offset.saturating_add(1);
                }
            }
            _ => {
                let max = self.current_list_length().saturating_sub(1);
                if self.list_offset < max {
                    self.list_offset = self.list_offset.saturating_add(1);
                }
            }
        }
    }

    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::OverviewSchedule => self.schedule_offset = self.schedule_offset.saturating_sub(1),
            Focus::OverviewHomework => self.homework_offset = self.homework_offset.saturating_sub(1),
            Focus::OverviewGrades => self.grades_offset = self.grades_offset.saturating_sub(1),
            _ => self.list_offset = self.list_offset.saturating_sub(1),
        }
    }

    /// Open the selected message thread
    pub fn open_thread(&mut self) -> Option<i64> {
        self.open_thread_at(self.list_offset)
    }

    /// Open a specific message thread by index
    pub fn open_thread_at(&mut self, index: usize) -> Option<i64> {
        if self.current_tab != Tab::Messages || self.message_view != MessageView::List {
            return None;
        }

        if let Some(thread) = self.messages.get(index) {
            let thread_id = thread.id;
            self.selected_thread_id = Some(thread_id);
            self.message_view = MessageView::Thread;
            self.thread_offset = 0;
            return Some(thread_id);
        }
        None
    }

    /// Close thread view and return to list
    pub fn close_thread(&mut self) {
        self.message_view = MessageView::List;
        self.selected_thread_id = None;
        self.thread_messages.clear();
        self.thread_offset = 0;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Start reply mode
    pub fn start_reply(&mut self) {
        if self.message_view == MessageView::Thread {
            self.input_mode = InputMode::Reply;
            self.input_buffer.clear();
            self.input_cursor = 0;
        }
    }

    /// Cancel input mode
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Add character to input buffer
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.insert(self.input_cursor, c);
        self.input_cursor += 1;
    }

    /// Delete character before cursor
    pub fn input_backspace(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    /// Delete character at cursor
    pub fn input_delete(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_buffer.remove(self.input_cursor);
        }
    }

    /// Move input cursor left
    pub fn input_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// Move input cursor right
    pub fn input_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor += 1;
        }
    }

    /// Get current input and clear buffer
    pub fn take_input(&mut self) -> String {
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.input_mode = InputMode::Normal;
        input
    }

    /// Start compose mode
    pub fn start_compose(&mut self) {
        self.message_view = MessageView::Compose;
        self.input_mode = InputMode::ComposeSubject;
        self.compose_subject.clear();
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.selected_recipients.clear();
    }

    /// Cancel compose and return to message list
    pub fn cancel_compose(&mut self) {
        self.message_view = MessageView::List;
        self.input_mode = InputMode::Normal;
        self.compose_subject.clear();
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.selected_recipients.clear();
    }

    /// Move to next compose step (subject -> body)
    pub fn compose_next_step(&mut self) {
        match self.input_mode {
            InputMode::ComposeSubject => {
                self.compose_subject = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.input_mode = InputMode::ComposeBody;
            }
            _ => {}
        }
    }

    /// Toggle recipient selection
    pub fn toggle_recipient(&mut self, index: usize) {
        if let Some(recipient) = self.recipients.get(index) {
            let id = recipient.id;
            if self.selected_recipients.contains(&id) {
                self.selected_recipients.retain(|&r| r != id);
            } else {
                self.selected_recipients.push(id);
            }
        }
    }

    /// Check if ready to send compose (has subject, body, and at least one recipient)
    pub fn can_send_compose(&self) -> bool {
        !self.compose_subject.is_empty()
            && !self.input_buffer.is_empty()
            && !self.selected_recipients.is_empty()
    }

    /// Activate the selected notification - navigate to the appropriate tab
    pub fn activate_notification(&mut self) -> bool {
        self.activate_notification_at(self.list_offset)
    }

    /// Activate a specific notification by index
    pub fn activate_notification_at(&mut self, index: usize) -> bool {
        if self.current_tab != Tab::Notifications {
            return false;
        }

        if let Some(notification) = self.notifications.get(index) {
            if let Some(ref notification_type) = notification.notification_type {
                let target_tab = match notification_type.as_str() {
                    "new_homework" => Some(Tab::Homework),
                    "new_grade" => Some(Tab::Grades),
                    "new_absence" => Some(Tab::Absences),
                    "new_feedback" | "new_badge" => Some(Tab::Feedbacks),
                    "new_event" | "new_event_reminder" => Some(Tab::Schedule),
                    "new_message" | "new_thread_message" => Some(Tab::Messages),
                    _ => None,
                };

                if let Some(tab) = target_tab {
                    self.current_tab = tab;
                    self.list_offset = 0;
                    self.focus = Focus::Content;
                    return true;
                }
            }
        }
        false
    }

    pub fn current_student(&self) -> Option<&StudentData> {
        self.students.get(self.selected_student)
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub async fn load_from_cache(&mut self, cache: &CacheStore) {
        // Load students
        if let Some((students, _, _)) = cache.get_students() {
            for student in students {
                let mut data = StudentData::new(student.clone());

                // Load homework
                if let Some((homework, age, _)) = cache.get_homework(student.id) {
                    data.homework = homework;
                    data.homework_age = Some(age);
                }

                // Load grades
                if let Some((grades, age, _)) = cache.get_grades(student.id) {
                    data.grades = grades;
                    data.grades_age = Some(age);
                }

                // Load schedule for today
                if let Some((schedule, age, _)) = cache.get_schedule(student.id, &self.current_date) {
                    data.schedule = schedule;
                    data.schedule_age = Some(age);
                }

                // Load events
                if let Some((events, _, _)) = cache.get_events(student.id) {
                    data.events = events;
                }

                // Load absences
                if let Some((absences, age, _)) = cache.get_absences(student.id) {
                    data.absences = absences;
                    data.absences_age = Some(age);
                }

                // Load feedbacks
                if let Some((feedbacks, age, _)) = cache.get_feedbacks(student.id) {
                    data.feedbacks = feedbacks;
                    data.feedbacks_age = Some(age);
                }

                self.students.push(data);
            }
        }

        // Load notifications
        if let Some((notifications, age, _)) = cache.get_notifications() {
            self.notifications = notifications;
            self.notifications_age = Some(age);
        }

        // Load messages
        if let Some((messages, age, _)) = cache.get_messages() {
            self.messages = messages;
            self.messages_age = Some(age);
        }
    }

    pub async fn refresh_data(&mut self, client: &ShkoloClient, cache: &CacheStore, force: bool) -> anyhow::Result<()> {
        self.loading = true;
        self.set_status("Refreshing...");

        // Fetch students
        let pupils_response = client.get_pupils().await?;

        let mut students = Vec::new();
        if let Some(child_pupils) = pupils_response.child_pupils {
            for (id, pupil) in child_pupils {
                students.push(Student::from_child_pupil(&id, &pupil));
            }
        }

        // Sort by name for consistent ordering
        students.sort_by(|a, b| a.name.cmp(&b.name));

        cache.save_students(&students)?;

        // Fetch data for each student
        self.students.clear();
        for student in &students {
            let mut data = StudentData::new(student.clone());

            // Check cache for homework
            let should_refresh_homework = force || cache.get_homework(student.id)
                .map(|(_, _, expired)| expired)
                .unwrap_or(true);

            if should_refresh_homework {
                if let Ok(homework) = self.fetch_homework(client, student.id).await {
                    data.homework = homework.clone();
                    data.homework_age = Some("just now".to_string());
                    let _ = cache.save_homework(student.id, &homework);
                }
            } else if let Some((homework, age, _)) = cache.get_homework(student.id) {
                data.homework = homework;
                data.homework_age = Some(age);
            }

            // Check cache for grades
            let should_refresh_grades = force || cache.get_grades(student.id)
                .map(|(_, _, expired)| expired)
                .unwrap_or(true);

            if should_refresh_grades {
                if let Ok(grades) = self.fetch_grades(client, student.id).await {
                    data.grades = grades.clone();
                    data.grades_age = Some("just now".to_string());
                    let _ = cache.save_grades(student.id, &grades);
                }
            } else if let Some((grades, age, _)) = cache.get_grades(student.id) {
                data.grades = grades;
                data.grades_age = Some(age);
            }

            // Check cache for schedule
            let should_refresh_schedule = force || cache.get_schedule(student.id, &self.current_date)
                .map(|(_, _, expired)| expired)
                .unwrap_or(true);

            if should_refresh_schedule {
                if let Ok(schedule) = self.fetch_schedule(client, student.id, &self.current_date).await {
                    data.schedule = schedule.clone();
                    data.schedule_age = Some("just now".to_string());
                    let _ = cache.save_schedule(student.id, &self.current_date, &schedule);
                }
            } else if let Some((schedule, age, _)) = cache.get_schedule(student.id, &self.current_date) {
                data.schedule = schedule;
                data.schedule_age = Some(age);
            }

            // Fetch events
            if let Ok(events_response) = client.get_pupil_events(student.id).await {
                let events: Vec<Event> = events_response.invitations
                    .unwrap_or_default()
                    .iter()
                    .map(Event::from_raw)
                    .collect();
                data.events = events.clone();
                let _ = cache.save_events(student.id, &events);
            }

            // Fetch absences
            let should_refresh_absences = force || cache.get_absences(student.id)
                .map(|(_, _, expired)| expired)
                .unwrap_or(true);

            if should_refresh_absences {
                if let Ok(absences) = self.fetch_absences(client, student.id).await {
                    data.absences = absences.clone();
                    data.absences_age = Some("just now".to_string());
                    let _ = cache.save_absences(student.id, &absences);
                }
            } else if let Some((absences, age, _)) = cache.get_absences(student.id) {
                data.absences = absences;
                data.absences_age = Some(age);
            }

            // Fetch feedbacks
            let should_refresh_feedbacks = force || cache.get_feedbacks(student.id)
                .map(|(_, _, expired)| expired)
                .unwrap_or(true);

            if should_refresh_feedbacks {
                if let Ok(feedbacks) = self.fetch_feedbacks(client, student.id).await {
                    data.feedbacks = feedbacks.clone();
                    data.feedbacks_age = Some("just now".to_string());
                    let _ = cache.save_feedbacks(student.id, &feedbacks);
                }
            } else if let Some((feedbacks, age, _)) = cache.get_feedbacks(student.id) {
                data.feedbacks = feedbacks;
                data.feedbacks_age = Some(age);
            }

            self.students.push(data);
        }

        // Fetch notifications
        let should_refresh_notifications = force || cache.get_notifications()
            .map(|(_, _, expired)| expired)
            .unwrap_or(true);

        if should_refresh_notifications {
            if let Ok(notifications) = self.fetch_notifications(client).await {
                self.notifications = notifications.clone();
                self.notifications_age = Some("just now".to_string());
                let _ = cache.save_notifications(&notifications);
            }
        } else if let Some((notifications, age, _)) = cache.get_notifications() {
            self.notifications = notifications;
            self.notifications_age = Some(age);
        }

        // Fetch messages (global, not per-student)
        let should_refresh_messages = force || cache.get_messages()
            .map(|(_, _, expired)| expired)
            .unwrap_or(true);

        if should_refresh_messages {
            if let Ok(messages) = self.fetch_messages(client).await {
                self.messages = messages.clone();
                self.messages_age = Some("just now".to_string());
                let _ = cache.save_messages(&messages);
            }
        } else if let Some((messages, age, _)) = cache.get_messages() {
            self.messages = messages;
            self.messages_age = Some(age);
        }

        self.last_refresh = Some({
            let now = OffsetDateTime::now_utc();
            format!("{:02}:{:02}", now.hour(), now.minute())
        });
        self.loading = false;
        self.clear_status();

        Ok(())
    }

    async fn fetch_homework(&self, client: &ShkoloClient, student_id: i64) -> anyhow::Result<Vec<Homework>> {
        let courses_response = client.get_homework_courses(student_id).await?;

        let mut all_homework = Vec::new();

        if let Some(courses) = courses_response.courses {
            let counts = courses_response.cyc_group_homeworks_count.unwrap_or_default();

            for course in courses {
                if let Some(cyc_group_id) = course.cyc_group_id {
                    let count = counts.get(&cyc_group_id.to_string()).copied().unwrap_or(0);
                    if count == 0 {
                        continue;
                    }

                    let subject = course.course_short_name
                        .or(course.course_name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    if let Ok(hw_response) = client.get_homework_list(cyc_group_id).await {
                        if let Some(homeworks) = hw_response.homeworks {
                            for item in homeworks {
                                all_homework.push(Homework::from_item(&item, &subject));
                            }
                        }
                    }
                }
            }
        }

        // Sort by date, newest first
        all_homework.sort_by(|a, b| {
            b.date_sort.cmp(&a.date_sort)
        });

        Ok(all_homework)
    }

    async fn fetch_grades(&self, client: &ShkoloClient, student_id: i64) -> anyhow::Result<Vec<Grade>> {
        let response = client.get_grades_summary(student_id).await?;

        let courses = response.grades.or(response.courses).unwrap_or_default();
        let grades: Vec<Grade> = courses
            .iter()
            .map(Grade::from_course_grades)
            .filter(|g| g.has_grades())
            .collect();

        Ok(grades)
    }

    async fn fetch_schedule(&self, client: &ShkoloClient, student_id: i64, date: &str) -> anyhow::Result<Vec<ScheduleHour>> {
        let response = client.get_pupil_schedule(student_id, date).await?;

        let hours = response.schedule_hours.or(response.data).unwrap_or_default();
        let mut schedule: Vec<ScheduleHour> = hours
            .iter()
            .map(ScheduleHour::from_raw)
            .collect();

        // Sort by hour number
        schedule.sort_by_key(|h| h.hour_number);

        Ok(schedule)
    }

    async fn fetch_notifications(&self, client: &ShkoloClient) -> anyhow::Result<Vec<Notification>> {
        let response = client.get_notifications(1).await?;

        let raw = response.data.or(response.notifications).unwrap_or_default();
        let notifications: Vec<Notification> = raw.iter()
            .map(Notification::from_raw)
            .collect();

        Ok(notifications)
    }

    async fn fetch_absences(&self, client: &ShkoloClient, student_id: i64) -> anyhow::Result<Vec<Absence>> {
        let response = client.get_absences(student_id).await?;

        let mut absences: Vec<Absence> = response.absences
            .unwrap_or_default()
            .iter()
            .map(Absence::from_raw)
            .collect();

        // Stable sort: by date (newest first), then by hour, then by subject for ties
        absences.sort_by(|a, b| {
            b.date_sort.cmp(&a.date_sort)
                .then_with(|| a.hour.cmp(&b.hour))
                .then_with(|| a.subject.cmp(&b.subject))
        });

        Ok(absences)
    }

    async fn fetch_feedbacks(&self, client: &ShkoloClient, student_id: i64) -> anyhow::Result<Vec<Feedback>> {
        let response = client.get_feedbacks(student_id).await?;

        let mut feedbacks: Vec<Feedback> = response.data
            .or(response.feedbacks)
            .unwrap_or_default()
            .iter()
            .map(Feedback::from_raw)
            .collect();

        // Stable sort: by date (newest first), then by subject for ties
        feedbacks.sort_by(|a, b| {
            b.date.cmp(&a.date)
                .then_with(|| a.subject.cmp(&b.subject))
        });

        Ok(feedbacks)
    }

    pub async fn fetch_messages(&self, client: &ShkoloClient) -> anyhow::Result<Vec<MessageThread>> {
        let raw_threads = client.get_messenger_threads(None).await?;

        let messages: Vec<MessageThread> = raw_threads.iter()
            .map(MessageThread::from_raw)
            .collect();

        Ok(messages)
    }

    /// Toggle the help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of clicking on a list item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickResult {
    None,
    StudentSelected,
    ItemSelected(usize),         // Item index selected
    ActivateNotification(usize), // Notification index to activate
    ActivateMessage(usize),      // Message index to open
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initial_state() {
        let app = App::new();

        assert!(app.running);
        assert_eq!(app.current_tab, Tab::Overview);
        assert_eq!(app.focus, Focus::Students);
        assert_eq!(app.selected_student, 0);
        assert!(!app.loading);
        assert!(!app.show_help);
    }

    #[test]
    fn test_students_pane_resize() {
        let mut app = App::new();
        let initial_width = app.students_pane_width;

        // Increase width
        app.resize_students_pane(5);
        assert_eq!(app.students_pane_width, initial_width + 5);

        // Decrease width
        app.resize_students_pane(-3);
        assert_eq!(app.students_pane_width, initial_width + 2);
    }

    #[test]
    fn test_students_pane_resize_bounds() {
        let mut app = App::new();

        // Try to shrink below minimum (15)
        app.students_pane_width = 20;
        app.resize_students_pane(-10);
        assert_eq!(app.students_pane_width, 15); // Clamped to min

        // Try to grow above maximum (60)
        app.students_pane_width = 55;
        app.resize_students_pane(10);
        assert_eq!(app.students_pane_width, 60); // Clamped to max
    }

    #[test]
    fn test_overview_split_resize() {
        let mut app = App::new();
        let initial_split = app.overview_split_percent;

        // Increase split
        app.resize_overview_split(10);
        assert_eq!(app.overview_split_percent, initial_split + 10);

        // Decrease split
        app.resize_overview_split(-5);
        assert_eq!(app.overview_split_percent, initial_split + 5);
    }

    #[test]
    fn test_overview_split_resize_bounds() {
        let mut app = App::new();

        // Try to shrink below minimum (20)
        app.overview_split_percent = 25;
        app.resize_overview_split(-10);
        assert_eq!(app.overview_split_percent, 20); // Clamped to min

        // Try to grow above maximum (70)
        app.overview_split_percent = 65;
        app.resize_overview_split(10);
        assert_eq!(app.overview_split_percent, 70); // Clamped to max
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = App::new();
        assert_eq!(app.current_tab, Tab::Overview);

        // Next tab (Overview -> Homework)
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Homework);

        // Previous tab (Homework -> Overview)
        app.prev_tab();
        assert_eq!(app.current_tab, Tab::Overview);

        // Previous tab wraps around (Overview -> Settings)
        app.prev_tab();
        assert_eq!(app.current_tab, Tab::Settings);
    }

    #[test]
    fn test_select_tab_by_index() {
        let mut app = App::new();
        assert_eq!(app.current_tab, Tab::Overview);

        // Select tab by index (0-8 for 9 tabs)
        app.select_tab(0);
        assert_eq!(app.current_tab, Tab::Overview);

        app.select_tab(1);
        assert_eq!(app.current_tab, Tab::Homework);

        app.select_tab(4);
        assert_eq!(app.current_tab, Tab::Absences);

        app.select_tab(8);
        assert_eq!(app.current_tab, Tab::Settings);

        // Invalid index should be ignored
        app.select_tab(99);
        assert_eq!(app.current_tab, Tab::Settings); // Unchanged
    }

    #[test]
    fn test_focus_toggle_on_overview() {
        let mut app = App::new();
        app.current_tab = Tab::Overview;
        assert_eq!(app.focus, Focus::Students);

        // Toggle cycles through: Students -> OverviewSchedule -> OverviewHomework -> OverviewGrades -> Students
        app.toggle_focus();
        assert_eq!(app.focus, Focus::OverviewSchedule);

        app.toggle_focus();
        assert_eq!(app.focus, Focus::OverviewHomework);

        app.toggle_focus();
        assert_eq!(app.focus, Focus::OverviewGrades);

        app.toggle_focus();
        assert_eq!(app.focus, Focus::Students);
    }

    #[test]
    fn test_student_selection() {
        let mut app = App::new();

        // Add mock students
        app.students = vec![
            StudentData::new(Student { id: 1, name: "Student 1".to_string(), class_name: None, school_name: None }),
            StudentData::new(Student { id: 2, name: "Student 2".to_string(), class_name: None, school_name: None }),
            StudentData::new(Student { id: 3, name: "Student 3".to_string(), class_name: None, school_name: None }),
        ];

        assert_eq!(app.selected_student, 0);

        // Next student
        app.next_student();
        assert_eq!(app.selected_student, 1);

        // Select by index
        app.select_student(2);
        assert_eq!(app.selected_student, 2);

        // Previous student
        app.prev_student();
        assert_eq!(app.selected_student, 1);
    }

    #[test]
    fn test_student_selection_bounds() {
        let mut app = App::new();

        app.students = vec![
            StudentData::new(Student { id: 1, name: "Student 1".to_string(), class_name: None, school_name: None }),
            StudentData::new(Student { id: 2, name: "Student 2".to_string(), class_name: None, school_name: None }),
        ];

        // Try to select beyond bounds - should be ignored
        app.selected_student = 0;
        app.select_student(5);
        assert_eq!(app.selected_student, 0); // Unchanged (invalid index ignored)

        // prev_student wraps around to last student
        app.selected_student = 0;
        app.prev_student();
        assert_eq!(app.selected_student, 1); // Wraps to last (index 1)
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new();
        assert!(!app.show_help);

        app.toggle_help();
        assert!(app.show_help);

        app.toggle_help();
        assert!(!app.show_help);
    }

    #[test]
    fn test_schedule_date_navigation() {
        let mut app = App::new();
        app.schedule_date = "2026-02-19".to_string();

        app.schedule_next_day();
        assert_eq!(app.schedule_date, "2026-02-20");

        app.schedule_prev_day();
        assert_eq!(app.schedule_date, "2026-02-19");
    }

    #[test]
    fn test_scroll_operations() {
        let mut app = App::new();
        // Need to be on a tab that uses list_offset with Content focus
        app.current_tab = Tab::Notifications;
        app.focus = Focus::Content;

        // Add some items so scrolling works
        app.notifications = vec![
            Notification { id: Some("1".into()), title: "N1".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: None, pupil_names: None },
            Notification { id: Some("2".into()), title: "N2".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: None, pupil_names: None },
            Notification { id: Some("3".into()), title: "N3".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: None, pupil_names: None },
        ];

        assert_eq!(app.list_offset, 0);

        app.scroll_down();
        assert_eq!(app.list_offset, 1);

        app.scroll_down();
        assert_eq!(app.list_offset, 2);

        app.scroll_up();
        assert_eq!(app.list_offset, 1);

        // Can't go below 0
        app.scroll_up();
        app.scroll_up();
        assert_eq!(app.list_offset, 0);
    }

    #[test]
    fn test_message_view_states() {
        let mut app = App::new();
        app.current_tab = Tab::Messages;

        assert_eq!(app.message_view, MessageView::List);

        // Add mock messages
        app.messages = vec![MessageThread {
            id: 1,
            subject: "Test".to_string(),
            last_message: "Preview".to_string(),
            last_sender: "Sender".to_string(),
            participant_count: 1,
            is_unread: true,
            updated_at: "19.02.2026".to_string(),
            creator: "Creator".to_string(),
        }];

        // Open thread
        app.list_offset = 0;
        let thread_id = app.open_thread();
        assert_eq!(thread_id, Some(1));
        assert_eq!(app.message_view, MessageView::Thread);

        // Close thread
        app.close_thread();
        assert_eq!(app.message_view, MessageView::List);
    }

    #[test]
    fn test_input_mode_operations() {
        let mut app = App::new();
        assert_eq!(app.input_mode, InputMode::Normal);

        // start_reply only works when in Thread view
        app.message_view = MessageView::Thread;
        app.start_reply();
        assert_eq!(app.input_mode, InputMode::Reply);
        assert!(app.input_buffer.is_empty());

        // Type some text
        app.input_char('H');
        app.input_char('i');
        assert_eq!(app.input_buffer, "Hi");
        assert_eq!(app.input_cursor, 2);

        // Move cursor
        app.input_left();
        assert_eq!(app.input_cursor, 1);

        // Backspace
        app.input_backspace();
        assert_eq!(app.input_buffer, "i");
        assert_eq!(app.input_cursor, 0);

        // Cancel input
        app.cancel_input();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn test_status_and_error_messages() {
        let mut app = App::new();

        // Set status
        app.set_status("Loading...");
        assert_eq!(app.status_message, Some("Loading...".to_string()));

        // Clear status
        app.clear_status();
        assert_eq!(app.status_message, None);

        // Set error
        app.set_error("Something went wrong");
        assert_eq!(app.error_message, Some("Something went wrong".to_string()));

        // Clear error
        app.clear_error();
        assert_eq!(app.error_message, None);
    }

    #[test]
    fn test_click_student_selection() {
        use crate::models::student::Student;

        let mut app = App::new();
        // Setup: 3 students, header_offset=3 (tabs + borders), students_width=25
        app.students = vec![
            StudentData::new(Student { id: 1, name: "Alice".into(), class_name: None, school_name: None }),
            StudentData::new(Student { id: 2, name: "Bob".into(), class_name: None, school_name: None }),
            StudentData::new(Student { id: 3, name: "Carol".into(), class_name: None, school_name: None }),
        ];
        let header_offset = 3;
        let students_width = 25;

        // Click on first student (row 4 = header 3 + border 1 + item 0)
        let result = app.click_list_item(4, header_offset, 5, students_width);
        assert!(matches!(result, ClickResult::StudentSelected));
        assert_eq!(app.selected_student, 0);

        // Click on second student (row 5)
        let result = app.click_list_item(5, header_offset, 5, students_width);
        assert!(matches!(result, ClickResult::StudentSelected));
        assert_eq!(app.selected_student, 1);

        // Click on third student (row 6)
        let result = app.click_list_item(6, header_offset, 5, students_width);
        assert!(matches!(result, ClickResult::StudentSelected));
        assert_eq!(app.selected_student, 2);

        // Click outside list bounds (row 7) - no change
        let result = app.click_list_item(7, header_offset, 5, students_width);
        assert!(matches!(result, ClickResult::None));
        assert_eq!(app.selected_student, 2); // Still last selected

        // Click in header area (row 3 = header_offset)
        let result = app.click_list_item(3, header_offset, 5, students_width);
        assert!(matches!(result, ClickResult::None));
    }

    #[test]
    fn test_click_content_does_not_scroll() {
        let mut app = App::new();
        app.current_tab = Tab::Notifications;
        app.focus = Focus::Content;

        // Setup notifications
        app.notifications = vec![
            Notification { id: Some("1".into()), title: "N1".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: Some("new_grade".into()), pupil_names: None },
            Notification { id: Some("2".into()), title: "N2".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: Some("new_homework".into()), pupil_names: None },
            Notification { id: Some("3".into()), title: "N3".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: Some("new_grade".into()), pupil_names: None },
            Notification { id: Some("4".into()), title: "N4".into(), body: Some("Body".into()), date: "".into(), is_read: false, notification_type: Some("new_grade".into()), pupil_names: None },
        ];

        let header_offset = 3;
        let students_width = 25;

        // Start scrolled down by 1
        app.list_offset = 1;
        let initial_offset = app.list_offset;

        // Click on visible item at row 4 (should be index 1 in visible area, so actual item index = 1 + 1 = 2)
        let result = app.click_list_item(4, header_offset, 30, students_width);

        // Click should return the correct index
        assert!(matches!(result, ClickResult::ActivateNotification(1)));

        // Scroll position should NOT have changed
        assert_eq!(app.list_offset, initial_offset);
    }

    #[test]
    fn test_click_notification_activates() {
        let mut app = App::new();
        app.current_tab = Tab::Notifications;
        app.focus = Focus::Content;

        app.notifications = vec![
            Notification { id: Some("1".into()), title: "N1".into(), body: None, date: "".into(), is_read: false, notification_type: Some("new_grade".into()), pupil_names: None },
            Notification { id: Some("2".into()), title: "N2".into(), body: None, date: "".into(), is_read: false, notification_type: Some("new_homework".into()), pupil_names: None },
        ];

        // Activate notification at index 1
        let success = app.activate_notification_at(1);
        assert!(success);
        assert_eq!(app.current_tab, Tab::Homework);

        // Reset
        app.current_tab = Tab::Notifications;

        // Activate notification at index 0
        let success = app.activate_notification_at(0);
        assert!(success);
        assert_eq!(app.current_tab, Tab::Grades);
    }

    #[test]
    fn test_click_message_opens_thread() {
        let mut app = App::new();
        app.current_tab = Tab::Messages;
        app.message_view = MessageView::List;

        app.messages = vec![
            MessageThread { id: 100, subject: "Thread A".into(), last_message: "".into(), last_sender: "".into(), participant_count: 1, is_unread: false, updated_at: "".into(), creator: "".into() },
            MessageThread { id: 200, subject: "Thread B".into(), last_message: "".into(), last_sender: "".into(), participant_count: 2, is_unread: true, updated_at: "".into(), creator: "".into() },
        ];

        // Open thread at index 1
        let result = app.open_thread_at(1);
        assert_eq!(result, Some(200));
        assert_eq!(app.message_view, MessageView::Thread);
        assert_eq!(app.selected_thread_id, Some(200));

        // Close and try index 0
        app.close_thread();
        let result = app.open_thread_at(0);
        assert_eq!(result, Some(100));
        assert_eq!(app.selected_thread_id, Some(100));
    }
}
