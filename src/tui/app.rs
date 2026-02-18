use crate::api::ShkoloClient;
use crate::cache::CacheStore;
use crate::models::*;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Homework,
    Grades,
    Schedule,
    Notifications,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Homework, Tab::Grades, Tab::Schedule, Tab::Notifications]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Homework => "Homework",
            Tab::Grades => "Grades",
            Tab::Schedule => "Schedule",
            Tab::Notifications => "Notifications",
        }
    }

    pub fn next(&self) -> Tab {
        match self {
            Tab::Overview => Tab::Homework,
            Tab::Homework => Tab::Grades,
            Tab::Grades => Tab::Schedule,
            Tab::Schedule => Tab::Notifications,
            Tab::Notifications => Tab::Overview,
        }
    }

    pub fn prev(&self) -> Tab {
        match self {
            Tab::Overview => Tab::Notifications,
            Tab::Homework => Tab::Overview,
            Tab::Grades => Tab::Homework,
            Tab::Schedule => Tab::Grades,
            Tab::Notifications => Tab::Schedule,
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
    pub homework_age: Option<String>,
    pub grades_age: Option<String>,
    pub schedule_age: Option<String>,
}

impl StudentData {
    pub fn new(student: Student) -> Self {
        Self {
            student,
            homework: Vec::new(),
            grades: Vec::new(),
            schedule: Vec::new(),
            events: Vec::new(),
            homework_age: None,
            grades_age: None,
            schedule_age: None,
        }
    }

    /// Get recent homework (last 2-3 days)
    pub fn recent_homework(&self) -> Vec<&Homework> {
        let now = OffsetDateTime::now_utc();
        let today = format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

        self.homework.iter()
            .filter(|hw| {
                hw.date_sort.as_ref()
                    .map(|d| d >= &today[..8].to_string() || d.starts_with(&today[..7]))
                    .unwrap_or(true)
            })
            .take(10)
            .collect()
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
    pub students: Vec<StudentData>,
    pub selected_student: usize,
    pub list_offset: usize,
    // Separate scroll offsets for overview sub-panes
    pub schedule_offset: usize,
    pub homework_offset: usize,
    pub grades_offset: usize,
    pub notifications: Vec<Notification>,
    pub notifications_age: Option<String>,
    pub status_message: Option<String>,
    pub loading: bool,
    pub last_refresh: Option<String>,
    pub current_date: String,
    pub current_time: (u8, u8), // (hour, minute)
}

impl App {
    pub fn new() -> Self {
        let now = OffsetDateTime::now_utc();
        let today = format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day());
        Self {
            running: true,
            current_tab: Tab::Overview,
            focus: Focus::Students,
            students: Vec::new(),
            selected_student: 0,
            list_offset: 0,
            schedule_offset: 0,
            homework_offset: 0,
            grades_offset: 0,
            notifications: Vec::new(),
            notifications_age: None,
            status_message: None,
            loading: false,
            last_refresh: None,
            current_date: today,
            current_time: (now.hour(), now.minute()),
        }
    }

    pub fn update_time(&mut self) {
        let now = OffsetDateTime::now_utc();
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

    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::OverviewSchedule => self.schedule_offset = self.schedule_offset.saturating_add(1),
            Focus::OverviewHomework => self.homework_offset = self.homework_offset.saturating_add(1),
            Focus::OverviewGrades => self.grades_offset = self.grades_offset.saturating_add(1),
            _ => self.list_offset = self.list_offset.saturating_add(1),
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

    pub fn current_student(&self) -> Option<&StudentData> {
        self.students.get(self.selected_student)
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
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

                self.students.push(data);
            }
        }

        // Load notifications
        if let Some((notifications, age, _)) = cache.get_notifications() {
            self.notifications = notifications;
            self.notifications_age = Some(age);
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
