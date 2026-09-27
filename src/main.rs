mod api;
mod cache;
mod holidays;
mod i18n;
mod models;
mod tui;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{Event, EventStream, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::io::{self, Write};
use std::time::Duration;
use time::OffsetDateTime;
use std::pin::Pin;
use std::future::Future;

use api::ShkoloClient;
use cache::CacheStore;
use i18n::{Lang, T};
use models::*;
use tui::{App, draw, handle_key, handlers::Action, app::{ClickResult, StudentData}};

/// Result of a background refresh operation
enum BackgroundResult {
    /// Full data refresh completed
    DataRefresh {
        students: Vec<StudentData>,
        notifications: Vec<Notification>,
        messages: Vec<MessageThread>,
        effective_date: String,
        /// Fresh usersAndYears entries (school contexts), if the call succeeded
        contexts: Option<Vec<UserYearEntry>>,
    },
    /// Schedule-only refresh completed
    ScheduleRefresh {
        student_id: i64,
        date: String,
        schedule: Vec<ScheduleHour>,
    },
}

const IOS_APP_STORAGE: &str = "Library/Containers/DD1CC5D9-F40E-415C-8E47-094321279222/Data/Library/Application Support/com.shkolo.mobileapp/RCTAsyncLocalStorage_V1/manifest.json";

#[derive(Parser)]
#[command(name = "shkolo")]
#[command(about = "CLI for Shkolo.bg - Bulgarian school management system")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Force refresh data from API
    #[arg(short, long, global = true)]
    refresh: bool,

    /// Bypass cache entirely
    #[arg(long, global = true)]
    no_cache: bool,

    /// Cache TTL in seconds (default: 3600)
    #[arg(long, global = true)]
    cache_ttl: Option<i64>,

    /// Language (bg or en)
    #[arg(long, global = true)]
    lang: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// JSON mode - output structured data for AI assistants
    Json {
        #[command(subcommand)]
        command: JsonCommands,

        /// Output format: pretty or compact
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Launch interactive TUI
    Tui,

    /// Import token from iOS Shkolo app
    ImportToken,

    /// Login with credentials
    Login {
        /// Username/email
        #[arg(short, long)]
        username: Option<String>,

        /// Password
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Login with Google
    LoginGoogle {
        /// Google ID token
        #[arg(long)]
        token: Option<String>,
    },

    /// Login with web cookie from app.shkolo.bg
    LoginWeb {
        /// The shkolo-cookie value from browser DevTools
        #[arg(long)]
        cookie: String,

        /// Optional XSRF-TOKEN value from browser DevTools
        #[arg(long)]
        xsrf_token: Option<String>,
    },

    /// Logout and clear token
    Logout,

    /// Show authentication status
    Status,

    /// Set default language (bg or en)
    SetLang {
        /// Language code: bg or en
        language: String,
    },

    /// Cache management
    Cache {
        /// Clear cache (keeps token)
        #[arg(long)]
        clear: bool,

        /// Clear all cache including token
        #[arg(long)]
        clear_all: bool,

        /// Force refresh all data
        #[arg(long)]
        refresh: bool,
    },
}

#[derive(Subcommand)]
enum JsonCommands {
    /// List students
    Students,

    /// Get homework
    Homework {
        /// Student name or index (optional, defaults to all)
        student: Option<String>,
    },

    /// Get grades
    Grades {
        /// Student name or index (optional, defaults to all)
        student: Option<String>,
    },

    /// Get schedule
    Schedule {
        /// Student name or index (optional, defaults to first)
        student: Option<String>,

        /// Date in YYYY-MM-DD format (defaults to today)
        date: Option<String>,
    },

    /// Get summary for all students
    Summary,

    /// Get absences
    Absences {
        /// Student name or index (optional, defaults to first)
        student: Option<String>,
    },

    /// Get feedbacks (badges/remarks)
    Feedbacks {
        /// Student name or index (optional, defaults to first)
        student: Option<String>,
    },

    /// Get notifications
    Notifications,

    /// Get messages
    Messages,

    /// Get a specific message thread (for debugging)
    Thread {
        /// Thread ID
        thread_id: i64,
    },

    /// Get raw feedbacks data (for debugging)
    FeedbacksRaw {
        /// Student name or index (optional, defaults to first)
        student: Option<String>,
    },

    /// List schools (parent user contexts) available to this account
    Schools {
        /// Include schools from previous school years
        #[arg(long)]
        all_years: bool,
    },

    /// Switch the active school (parent user) and school year
    SwitchSchool {
        /// Parent user id, as listed by `json schools`
        #[arg(long)]
        user_id: i64,

        /// School year id (defaults to the latest year for that user)
        #[arg(long)]
        year: Option<i64>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get cache TTL from env, config, or default
    let ttl = cli.cache_ttl
        .or_else(|| std::env::var("SHKOLO_CACHE_TTL").ok().and_then(|v| v.parse().ok()));

    let cache = CacheStore::new(ttl)?;

    // Resolve language: --lang flag > persisted ui_config > SHKOLO_LANG env > default (Bg)
    let lang = if let Some(ref lang_str) = cli.lang {
        Lang::from_str(lang_str).unwrap_or_default()
    } else {
        let ui_config = cache.load_ui_config();
        if let Some(ref lang_str) = ui_config.language {
            Lang::from_str(lang_str).unwrap_or_default()
        } else if let Ok(env_lang) = std::env::var("SHKOLO_LANG") {
            Lang::from_str(&env_lang).unwrap_or_default()
        } else {
            Lang::default()
        }
    };

    match cli.command {
        Commands::Json { command, format } => {
            run_json_command(command, &cache, cli.refresh, cli.no_cache, &format).await
        }
        Commands::Tui => run_tui(&cache, lang).await,
        Commands::ImportToken => import_token(&cache, lang).await,
        Commands::Login { username, password } => login(&cache, username, password, lang).await,
        Commands::LoginGoogle { token } => login_google(&cache, token, lang).await,
        Commands::LoginWeb { cookie, xsrf_token } => login_web(&cache, &cookie, xsrf_token.as_deref(), lang).await,
        Commands::Logout => logout(&cache, lang).await,
        Commands::Status => show_status(&cache, lang),
        Commands::SetLang { language } => set_lang(&cache, &language),
        Commands::Cache { clear, clear_all, refresh } => {
            cache_command(&cache, clear, clear_all, refresh, lang).await
        }
    }
}

async fn run_json_command(
    command: JsonCommands,
    cache: &CacheStore,
    force_refresh: bool,
    no_cache: bool,
    format: &str,
) -> Result<()> {
    let client = get_authenticated_client(cache)?;

    match command {
        JsonCommands::Students => {
            let (students, cached, cached_at) = get_students(&client, cache, force_refresh || no_cache).await?;
            output_json(&api::ApiResponse::new(students, cached && !no_cache, cached_at), format)?;
        }
        JsonCommands::Homework { student } => {
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            let mut all_homework = Vec::new();
            let mut any_cached = false;
            let mut oldest_cache: Option<String> = None;

            for s in selected {
                ensure_student_context(&client, cache, s).await?;
                let (homework, cached, cached_at) = get_homework(&client, cache, s.id, force_refresh || no_cache).await?;
                if cached {
                    any_cached = true;
                    if oldest_cache.is_none() {
                        oldest_cache = cached_at;
                    }
                }
                all_homework.push(serde_json::json!({
                    "student": s,
                    "homework": homework,
                }));
            }

            output_json(&api::ApiResponse::new(all_homework, any_cached && !no_cache, oldest_cache), format)?;
        }
        JsonCommands::Grades { student } => {
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            let mut all_grades = Vec::new();
            let mut any_cached = false;
            let mut oldest_cache: Option<String> = None;

            for s in selected {
                ensure_student_context(&client, cache, s).await?;
                let (grades, cached, cached_at) = get_grades(&client, cache, s.id, force_refresh || no_cache).await?;
                if cached {
                    any_cached = true;
                    if oldest_cache.is_none() {
                        oldest_cache = cached_at;
                    }
                }
                all_grades.push(serde_json::json!({
                    "student": s,
                    "grades": grades,
                }));
            }

            output_json(&api::ApiResponse::new(all_grades, any_cached && !no_cache, oldest_cache), format)?;
        }
        JsonCommands::Schedule { student, date } => {
            let date = date.unwrap_or_else(|| get_today_date());
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            let mut all_schedules = Vec::new();
            let mut any_cached = false;
            let mut oldest_cache: Option<String> = None;

            for s in selected {
                ensure_student_context(&client, cache, s).await?;
                let (schedule, cached, cached_at) = get_schedule(&client, cache, s.id, &date, force_refresh || no_cache).await?;
                if cached {
                    any_cached = true;
                    if oldest_cache.is_none() {
                        oldest_cache = cached_at;
                    }
                }
                all_schedules.push(serde_json::json!({
                    "student": s,
                    "date": date,
                    "holiday": holidays::public_holiday_on(&date).map(|h| serde_json::json!({
                        "name_bg": h.name_bg,
                        "name_en": h.name_en,
                    })),
                    "schedule": schedule,
                }));
            }

            output_json(&api::ApiResponse::new(all_schedules, any_cached && !no_cache, oldest_cache), format)?;
        }
        JsonCommands::Summary => {
            let date = get_today_date();
            let (students, students_cached, _) = get_students(&client, cache, force_refresh || no_cache).await?;

            let mut summaries = Vec::new();

            for s in &students {
                ensure_student_context(&client, cache, s).await?;
                let (homework, _, _) = get_homework(&client, cache, s.id, force_refresh || no_cache).await?;
                let (grades, _, _) = get_grades(&client, cache, s.id, force_refresh || no_cache).await?;
                let (schedule, _, _) = get_schedule(&client, cache, s.id, &date, force_refresh || no_cache).await?;

                // Get recent homework (last 5)
                let recent_homework: Vec<_> = homework.into_iter().take(5).collect();

                summaries.push(serde_json::json!({
                    "student": s,
                    "today_schedule": schedule,
                    "recent_homework": recent_homework,
                    "grades_count": grades.len(),
                }));
            }

            output_json(&api::ApiResponse::new(summaries, students_cached && !no_cache, None), format)?;
        }
        JsonCommands::Absences { student } => {
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            let mut all_absences = Vec::new();
            let mut any_cached = false;
            let mut oldest_cache: Option<String> = None;

            for s in selected {
                ensure_student_context(&client, cache, s).await?;
                let (absences, cached, cached_at) = get_absences(&client, cache, s.id, force_refresh || no_cache).await?;
                if cached {
                    any_cached = true;
                    if oldest_cache.is_none() {
                        oldest_cache = cached_at;
                    }
                }
                all_absences.push(serde_json::json!({
                    "student": s,
                    "absences": absences,
                    "total": absences.len(),
                    "excused": absences.iter().filter(|a| a.is_excused).count(),
                    "unexcused": absences.iter().filter(|a| !a.is_excused).count(),
                }));
            }

            output_json(&api::ApiResponse::new(all_absences, any_cached && !no_cache, oldest_cache), format)?;
        }
        JsonCommands::Feedbacks { student } => {
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            let mut all_feedbacks = Vec::new();
            let mut any_cached = false;
            let mut oldest_cache: Option<String> = None;

            for s in selected {
                ensure_student_context(&client, cache, s).await?;
                let (feedbacks, cached, cached_at) = get_feedbacks(&client, cache, s.id, force_refresh || no_cache).await?;
                if cached {
                    any_cached = true;
                    if oldest_cache.is_none() {
                        oldest_cache = cached_at;
                    }
                }
                all_feedbacks.push(serde_json::json!({
                    "student": s,
                    "feedbacks": feedbacks,
                    "total": feedbacks.len(),
                    "positive": feedbacks.iter().filter(|f| f.is_positive).count(),
                    "negative": feedbacks.iter().filter(|f| !f.is_positive).count(),
                }));
            }

            output_json(&api::ApiResponse::new(all_feedbacks, any_cached && !no_cache, oldest_cache), format)?;
        }
        JsonCommands::Notifications => {
            let (notifications, cached, cached_at) = get_notifications(&client, cache, force_refresh || no_cache).await?;

            let unread = notifications.iter().filter(|n| !n.is_read).count();
            output_json(&api::ApiResponse::new(serde_json::json!({
                "notifications": notifications,
                "total": notifications.len(),
                "unread": unread,
            }), cached && !no_cache, cached_at), format)?;
        }
        JsonCommands::Messages => {
            // Use the correct messenger API
            let mut results = serde_json::json!({});

            // Get folders
            match client.get_messenger_folders().await {
                Ok(data) => results["folders"] = serde_json::to_value(data)?,
                Err(e) => results["folders_error"] = serde_json::json!(e.to_string()),
            }

            // Get threads (inbox)
            match client.get_messenger_threads(None).await {
                Ok(data) => results["threads"] = serde_json::to_value(data)?,
                Err(e) => results["threads_error"] = serde_json::json!(e.to_string()),
            }

            // Check if can send
            match client.can_send_messages().await {
                Ok(data) => results["can_send"] = serde_json::json!(data),
                Err(e) => results["can_send_error"] = serde_json::json!(e.to_string()),
            }

            output_json(&api::ApiResponse::new(results, false, None), format)?;
        }
        JsonCommands::Schools { all_years } => {
            let response = client.get_users_and_years().await?;
            let contexts = SchoolContext::from_entries(response.users.as_deref().unwrap_or(&[]));
            let current = SchoolContext::current_year(&contexts);
            let active = cache.load_token().ok().and_then(|t| Some((t.user_id?, t.school_year?)));

            let schools: Vec<serde_json::Value> = contexts.iter()
                .filter(|c| all_years || c.is_current(current))
                .map(|c| serde_json::json!({
                    "user_id": c.user_id,
                    "school_year": c.school_year,
                    "year_label": c.year_label(),
                    "school_name": c.school_name,
                    "user_name": c.user_name,
                    "is_current_year": c.is_current(current),
                    "active": Some(c.key()) == active || (active.is_none() && c.is_selected),
                }))
                .collect();

            output_json(&api::ApiResponse::new(serde_json::json!({
                "current_school_year": current,
                "schools": schools,
            }), false, None), format)?;
        }
        JsonCommands::SwitchSchool { user_id, year } => {
            let year = match year {
                Some(y) => y,
                None => {
                    let response = client.get_users_and_years().await?;
                    response.users.iter().flatten()
                        .filter(|e| e.id == Some(user_id))
                        .filter_map(|e| e.school_year)
                        .max()
                        .ok_or_else(|| anyhow!("Unknown user id {}", user_id))?
                }
            };

            let users = client.switch_user_and_year(user_id, year, None).await?;
            persist_context_switch(cache, &client, &users, user_id, year);

            let school_name = users.users.iter().flatten()
                .find(|e| e.id == Some(user_id) && e.school_year == Some(year))
                .and_then(|e| e.school_name.clone());

            output_json(&api::ApiResponse::new(serde_json::json!({
                "switched": true,
                "user_id": user_id,
                "school_year": year,
                "year_label": SchoolContext::year_label_for(year),
                "school_name": school_name,
                "token_changed": cache.load_token().map(|t| Some(t.token) != client.token()).unwrap_or(false),
            }), false, None), format)?;
        }
        JsonCommands::Thread { thread_id } => {
            // Get raw thread data for debugging, in the school the thread was listed under
            let context = cache.get_messages()
                .and_then(|(threads, _, _)| thread_context(&threads, thread_id));
            match with_thread_context(&client, cache, context, || client.get_thread_raw(thread_id)).await {
                Ok((data, _)) => output_json(&api::ApiResponse::new(data, false, None), format)?,
                Err(e) => output_json(&api::ApiResponse::new(serde_json::json!({
                    "error": e.to_string(),
                    "thread_id": thread_id,
                }), false, None), format)?,
            }
        }
        JsonCommands::FeedbacksRaw { student } => {
            let (students, _, _) = get_students(&client, cache, force_refresh || no_cache).await?;
            let selected = select_students(&students, student.as_deref());

            if let Some(s) = selected.first() {
                ensure_student_context(&client, cache, s).await?;
                match client.get_feedbacks_raw(s.id).await {
                    Ok(data) => output_json(&api::ApiResponse::new(data, false, None), format)?,
                    Err(e) => output_json(&api::ApiResponse::new(serde_json::json!({
                        "error": e.to_string(),
                        "student_id": s.id,
                    }), false, None), format)?,
                }
            } else {
                output_json(&api::ApiResponse::new(serde_json::json!({
                    "error": "No students found",
                }), false, None), format)?;
            }
        }
    }

    Ok(())
}

async fn run_tui(cache: &CacheStore, lang: Lang) -> Result<()> {
    let client = get_authenticated_client(cache)?;

    // Create app (before raw mode so we can print errors)
    let mut app = App::new();
    app.lang = lang;

    // Load user name from token cache
    if let Ok(token_data) = cache.load_token() {
        if let Some(ref data) = token_data.user_data {
            if let Some(names) = data.get("names").and_then(|v| v.as_str()) {
                app.user_name = Some(names.to_string());
            }
            if app.user_name.is_none() {
                if let Some(users) = data.get("users").and_then(|v| v.as_array()) {
                    if let Some(first) = users.first() {
                        if let Some(n) = first.get("user_name").and_then(|v| v.as_str()) {
                            app.user_name = Some(n.to_string());
                        }
                    }
                }
            }
        }
    }

    // School contexts (one per school per year) from the token file; refreshed in the background
    if let Ok(token_data) = cache.load_token() {
        if let (Some(user_id), Some(year)) = (token_data.user_id, token_data.school_year) {
            app.active_context = Some((user_id, year));
        }
        app.set_school_contexts(contexts_from_token(&token_data));
    }

    // Load UI configuration (pane sizes, etc.)
    let ui_config = cache.load_ui_config();
    if let Some(width) = ui_config.students_pane_width {
        app.students_pane_width = width;
    }
    if let Some(percent) = ui_config.overview_split_percent {
        app.overview_split_percent = percent;
    }

    // Setup terminal with mouse support
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load cached data first
    app.load_from_cache(cache).await;

    // Main loop - async event handling with background refresh
    let mut last_time_update = std::time::Instant::now();
    let mut last_auto_refresh = std::time::Instant::now();
    let mut event_stream = EventStream::new();

    // Type alias for background task
    type BackgroundTask = Pin<Box<dyn Future<Output = Result<BackgroundResult>> + Send>>;

    // Initial refresh on startup
    app.loading = true;
    app.set_status(T::loading(app.lang));
    let mut background_task: Option<BackgroundTask> = {
        let client_clone = client.clone();
        let cache_clone = cache.clone();
        let student_ids: Vec<i64> = app.students.iter().map(|s| s.student.id).collect();
        Some(Box::pin(async move {
            refresh_data_background(&client_clone, &cache_clone, false, student_ids).await
        }))
    };

    loop {
        // Update time periodically for schedule highlighting (once per minute is enough)
        if last_time_update.elapsed() >= Duration::from_secs(60) {
            app.update_time();
            last_time_update = std::time::Instant::now();
        }

        // Auto-refresh check
        if background_task.is_none() {
            if let Some(interval_minutes) = app.auto_refresh_interval.minutes() {
                let interval_secs = interval_minutes * 60;
                if last_auto_refresh.elapsed() >= Duration::from_secs(interval_secs) {
                    app.loading = true;
                    app.set_status(T::auto_refreshing(app.lang));
                    let client_clone = client.clone();
                    let cache_clone = cache.clone();
                    let student_ids: Vec<i64> = app.students.iter().map(|s| s.student.id).collect();
                    background_task = Some(Box::pin(async move {
                        refresh_data_background(&client_clone, &cache_clone, false, student_ids).await
                    }));
                    last_auto_refresh = std::time::Instant::now();
                }
            }
        }

        // Tick for loading animation
        if app.loading {
            app.tick();
        }

        terminal.draw(|f| draw(f, &app))?;

        // Use tokio::select! to handle events and background tasks concurrently
        // Short tick for responsive input handling; slightly longer when not loading
        let tick_delay = tokio::time::sleep(Duration::from_millis(if app.loading { 50 } else { 100 }));

        tokio::select! {
            // Handle background task completion
            result = async {
                match &mut background_task {
                    Some(task) => Some(task.await),
                    None => {
                        std::future::pending::<()>().await;
                        None
                    }
                }
            } => {
                background_task = None;
                app.loading = false;

                if let Some(Ok(bg_result)) = result {
                    match bg_result {
                        BackgroundResult::DataRefresh { students, notifications, messages, effective_date, contexts } => {
                            app.students = students;
                            app.notifications = notifications;
                            app.messages = messages;
                            app.effective_date = effective_date;
                            if let Some(entries) = contexts {
                                app.set_school_contexts(SchoolContext::from_entries(&entries));
                                if let Ok(mut token_data) = cache.load_token() {
                                    if let Ok(json) = serde_json::to_value(&entries) {
                                        store_users_in_token(&mut token_data, json);
                                        let _ = cache.save_token_full(&token_data);
                                    }
                                }
                            }
                            // Reset schedule_date to effective date (today or next school day)
                            app.schedule_today();
                            app.set_status("Refreshed");
                        }
                        BackgroundResult::ScheduleRefresh { student_id, date, schedule } => {
                            // Update schedule for the specific student
                            if let Some(student_data) = app.students.iter_mut().find(|s| s.student.id == student_id) {
                                student_data.schedule = schedule;
                            }
                            app.set_status(format!("{} {}", T::loaded(app.lang), date));
                        }
                    }
                } else if let Some(Err(e)) = result {
                    app.set_status(format!("{} {}", T::error_prefix(app.lang), e));
                }
            }

            // Tick for animation
            _ = tick_delay => {}

            // Handle terminal events
            maybe_event = event_stream.next() => {
                if let Some(Ok(event)) = maybe_event {
                    match event {
                        Event::Key(key) => {
                            let action = handle_key(&mut app, key);
                            match action {
                                Action::Refresh if background_task.is_none() => {
                                    app.loading = true;
                                    app.set_status("Refreshing...");
                                    let client_clone = client.clone();
                                    let cache_clone = cache.clone();
                                    let student_ids: Vec<i64> = app.students.iter().map(|s| s.student.id).collect();
                                    background_task = Some(Box::pin(async move {
                                        refresh_data_background(&client_clone, &cache_clone, false, student_ids).await
                                    }));
                                    last_auto_refresh = std::time::Instant::now();
                                }
                                Action::RefreshAll if background_task.is_none() => {
                                    app.loading = true;
                                    app.set_status("Refreshing all...");
                                    let client_clone = client.clone();
                                    let cache_clone = cache.clone();
                                    let student_ids: Vec<i64> = app.students.iter().map(|s| s.student.id).collect();
                                    background_task = Some(Box::pin(async move {
                                        refresh_data_background(&client_clone, &cache_clone, true, student_ids).await
                                    }));
                                    last_auto_refresh = std::time::Instant::now();
                                }
                                Action::RefreshSchedule if background_task.is_none() => {
                                    app.loading = true;
                                    let schedule_date = app.schedule_date.clone();
                                    let student_id = app.current_student().map(|s| s.student.id);
                                    app.set_status(format!("{} {}...", T::loading_base(app.lang), schedule_date));
                                    if let Some(sid) = student_id {
                                        let client_clone = client.clone();
                                        let cache_clone = cache.clone();
                                        background_task = Some(Box::pin(async move {
                                            refresh_schedule_background(&client_clone, &cache_clone, sid, &schedule_date).await
                                        }));
                                    }
                                }
                                Action::Logout => {
                                    // Clear token and exit
                                    if let Err(e) = cache.clear_token() {
                                        app.set_status(format!("{} {}", T::logout_error(app.lang), e));
                                    } else {
                                        app.set_status(T::logged_out(app.lang));
                                        app.user_name = None;
                                        // Exit after logout
                                        app.quit();
                                    }
                                }
                                Action::OpenThread(thread_id) => {
                                    // Load thread messages
                                    app.loading = true;
                                    app.set_status(T::loading_thread(app.lang));
                                    terminal.draw(|f| draw(f, &app))?;

                                    match load_thread_messages(&client, cache, &mut app, thread_id).await {
                                        Ok(messages) => {
                                            app.thread_messages = messages;
                                            app.loading = false;
                                            app.clear_status();
                                        }
                                        Err(e) => {
                                            app.set_error(format!("{}\n{}", T::failed_load_thread(app.lang), e));
                                            app.loading = false;
                                            app.close_thread();
                                        }
                                    }
                                }
                                Action::CloseThread => {
                                    // Already handled in app.close_thread()
                                }
                                Action::SendReply(message) => {
                                    if let Some(thread_id) = app.selected_thread_id {
                                        app.loading = true;
                                        app.set_status(T::sending(app.lang));
                                        terminal.draw(|f| draw(f, &app))?;

                                        match send_thread_reply(&client, cache, &mut app, thread_id, &message).await {
                                            Ok(_) => {
                                                // Reload thread messages
                                                match load_thread_messages(&client, cache, &mut app, thread_id).await {
                                                    Ok(messages) => {
                                                        app.thread_messages = messages;
                                                        app.set_status(T::message_sent(app.lang));
                                                    }
                                                    Err(e) => {
                                                        app.set_status(format!("{} {}", T::sent_reload_failed(app.lang), e));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                app.set_status(format!("{} {}", T::send_failed(app.lang), e));
                                            }
                                        }
                                        app.loading = false;
                                    }
                                }
                                Action::StartCompose => {
                                    // Fetch recipients
                                    app.loading = true;
                                    app.set_status(T::loading_recipients(app.lang));
                                    terminal.draw(|f| draw(f, &app))?;

                                    match client.get_recipients().await {
                                        Ok(recipients) => {
                                            app.recipients = recipients;
                                            app.loading = false;
                                            app.clear_status();
                                        }
                                        Err(e) => {
                                            app.set_status(format!("{} {}", T::error_prefix(app.lang), e));
                                            app.loading = false;
                                            app.cancel_compose();
                                        }
                                    }
                                }
                                Action::SendCompose { subject, body, recipients } => {
                                    app.loading = true;
                                    app.set_status(T::sending_message(app.lang));
                                    terminal.draw(|f| draw(f, &app))?;

                                    match client.create_thread(&recipients, &subject, &body).await {
                                        Ok(_) => {
                                            app.set_status(T::message_sent(app.lang));
                                            // Refresh messages list (all schools)
                                            if let Ok((messages, _, _)) = get_messages(&client, cache, true).await {
                                                app.messages = messages;
                                            }
                                        }
                                        Err(e) => {
                                            app.set_status(format!("{} {}", T::send_failed(app.lang), e));
                                        }
                                    }
                                    app.loading = false;
                                }
                                Action::NavigateBack | Action::NavigateForward => {
                                    // Navigation was handled in the handler, but we may need
                                    // to reload thread messages if we navigated to a thread view
                                    if app.message_view == tui::app::MessageView::Thread {
                                        if let Some(thread_id) = app.selected_thread_id {
                                            app.loading = true;
                                            app.set_status(T::loading_thread(app.lang));
                                            terminal.draw(|f| draw(f, &app))?;

                                            match load_thread_messages(&client, cache, &mut app, thread_id).await {
                                                Ok(messages) => {
                                                    app.thread_messages = messages;
                                                    app.loading = false;
                                                    app.clear_status();
                                                }
                                                Err(e) => {
                                                    app.set_error(format!("{}\n{}", T::failed_load_thread(app.lang), e));
                                                    app.loading = false;
                                                    app.close_thread();
                                                }
                                            }
                                        }
                                    }
                                }
                                Action::None => {}
                                // These are handled by guards above (when background_task.is_none())
                                // If we get here, a background task is already running
                                Action::Refresh | Action::RefreshAll | Action::RefreshSchedule => {
                                    // Already refreshing, ignore
                                }
                            }
                        }
                        Event::Mouse(mouse) => {
                            match mouse.kind {
                                // Mouse scroll wheel
                                MouseEventKind::ScrollUp => {
                                    app.scroll_up();
                                }
                                MouseEventKind::ScrollDown => {
                                    app.scroll_down();
                                }
                                // Mouse click / drag start
                                MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                    let terminal_size = terminal.size()?;
                                    let header_height = 3u16;
                                    let footer_height = 3u16;
                                    let content_height = terminal_size.height.saturating_sub(header_height + footer_height);
                                    let content_area = (0, header_height, terminal_size.width, content_height);

                                    // First check if starting a drag on a border
                                    if app.start_drag(mouse.row, mouse.column, content_area) {
                                        // Drag started, don't process as click
                                    }
                                    // Tab bar is at rows 0-2 (3-line header with border)
                                    else if mouse.row <= 2 {
                                        app.click_tab(mouse.column);
                                    }
                                    // Content area starts at row 3
                                    else if mouse.row >= 3 {
                                        // Check if click is in content area (not status bar)
                                        let content_bottom = terminal_size.height.saturating_sub(3);
                                        if mouse.row < content_bottom {
                                            let content_height = content_bottom.saturating_sub(3); // from row 3 to status bar
                                            let click_result = app.click_list_item(mouse.row, 3, mouse.column, app.effective_students_width(), content_height);
                                            match click_result {
                                                ClickResult::ActivateNotification(index) => {
                                                    app.activate_notification_at(index);
                                                }
                                                ClickResult::ActivateMessage(index) => {
                                                    if let Some(thread_id) = app.open_thread_at(index) {
                                                        // Load thread messages
                                                        app.set_status(T::loading_thread(app.lang));
                                                        match load_thread_messages(&client, cache, &mut app, thread_id).await {
                                                            Ok(messages) => {
                                                                app.thread_messages = messages;
                                                                app.clear_status();
                                                            }
                                                            Err(e) => {
                                                                app.set_error(format!("{}\n{}", T::failed_load_thread(app.lang), e));
                                                                app.close_thread();
                                                            }
                                                        }
                                                    }
                                                }
                                                ClickResult::ItemSelected(_) | ClickResult::StudentSelected | ClickResult::None => {}
                                            }
                                        }
                                    }
                                }
                                // Drag for split resizing
                                MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                                    if app.is_dragging() {
                                        let terminal_size = terminal.size()?;
                                        let header_height = 3u16;
                                        let footer_height = 3u16;
                                        let content_height = terminal_size.height.saturating_sub(header_height + footer_height);
                                        let content_area = (0, header_height, terminal_size.width, content_height);
                                        app.update_drag(mouse.row, mouse.column, content_area);
                                    }
                                }
                                // End drag on mouse up
                                MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                                    app.end_drag();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        } // end tokio::select!

        if !app.running {
            break;
        }
    }

    // Save UI configuration (pane sizes + language)
    let ui_config = cache::UiConfig {
        students_pane_width: Some(app.students_pane_width),
        overview_split_percent: Some(app.overview_split_percent),
        language: Some(app.lang.as_str().to_string()),
    };
    let _ = cache.save_ui_config(&ui_config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;

    Ok(())
}

async fn import_token(cache: &CacheStore, lang: Lang) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let ios_path = home.join(IOS_APP_STORAGE);

    if !ios_path.exists() {
        eprintln!("Error: Shkolo iOS app data not found at:");
        eprintln!("  {}", ios_path.display());
        eprintln!();
        eprintln!("Make sure the Shkolo app is installed and you've logged in.");
        return Err(anyhow!("iOS app data not found"));
    }

    let content = std::fs::read_to_string(&ios_path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    let token = data.get("@ShkoloStore:Token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("No token found in app storage"))?;

    let user_name = data.get("@ShkoloStore:CurrentUserNames")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let user_id = data.get("@ShkoloStore:CurrentUserId")
        .and_then(|v| v.as_str());

    cache.save_token(token, None, user_id.and_then(|id| id.parse().ok()), Some(serde_json::json!({
        "names": user_name,
        "id": user_id,
    })))?;

    println!("{}", T::cli_token_imported(lang));
    println!("{}: {}", T::cli_user(lang), user_name);
    if let Some(id) = user_id {
        println!("User ID: {}", id);
    }

    // Fetch the school contexts so status and the TUI know the active school right away
    let client = ShkoloClient::with_token(token.to_string(), None);
    match client.get_users_and_years().await {
        Ok(users) => {
            let selected = users.users.iter().flatten().find(|e| e.is_selected == Some(true));
            if let Ok(mut token_data) = cache.load_token() {
                token_data.user_id = selected.and_then(|e| e.id).or_else(|| user_id.and_then(|s| s.parse().ok()));
                token_data.school_year = selected.and_then(|e| e.school_year);
                if let Ok(json) = serde_json::to_value(&users) {
                    store_users_in_token(&mut token_data, json);
                }
                cache.save_token_full(&token_data)?;
            }

            let contexts = SchoolContext::from_entries(users.users.as_deref().unwrap_or(&[]));
            let current = SchoolContext::current_year(&contexts);
            let current_schools: Vec<&SchoolContext> = contexts.iter().filter(|c| c.is_current(current)).collect();
            if let Some(active) = current_schools.iter().find(|c| c.is_selected) {
                println!("{}: {} ({})", T::cli_school(lang), active.school_name, active.year_label());
            }
            if current_schools.len() > 1 {
                println!("{}:", T::cli_available_schools(lang));
                for c in current_schools {
                    println!("  {} {} [user {}]", if c.is_selected { "*" } else { " " }, c.school_name, c.user_id);
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: could not fetch schools ({}). Run 'shkolo status' after the first TUI refresh.", e);
        }
    }

    Ok(())
}

async fn login(cache: &CacheStore, username: Option<String>, password: Option<String>, lang: Lang) -> Result<()> {
    let username = match username {
        Some(u) => u,
        None => {
            print!("Username: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let password = match password {
        Some(p) => p,
        None => {
            print!("Password: ");
            io::stdout().flush()?;
            rpassword::read_password()?
        }
    };

    let mut client = ShkoloClient::new();
    let response = client.login(&username, &password).await?;

    // Save token
    let user_data = serde_json::to_value(&response)?;
    cache.save_token(
        &client.token().unwrap(),
        client.school_year(),
        selected_user_id(&response),
        Some(user_data.clone()),
    )?;

    println!("{}", T::cli_logged_in(lang));

    if let Some(entries) = response.users {
        // Show user name from first entry
        if let Some(entry) = entries.first() {
            if let Some(ref name) = entry.user_name {
                println!("  {}: {}", T::cli_user(lang), name);
            }
            if let Some(ref roles) = entry.roles {
                if let Some(roles_str) = roles.as_str() {
                    println!("  {}: {}", T::cli_roles(lang), roles_str);
                }
            }
        }

        // Show the schools selectable for the current school year
        let contexts = SchoolContext::from_entries(&entries);
        let current = SchoolContext::current_year(&contexts);
        let current_schools: Vec<&SchoolContext> = contexts.iter().filter(|c| c.is_current(current)).collect();
        if current_schools.len() > 1 {
            println!("  {}:", T::cli_available_schools(lang));
            for c in current_schools {
                println!("    {} {} ({})", if c.is_selected { "*" } else { " " }, c.school_name, c.year_label());
            }
        }
    }

    Ok(())
}

async fn login_google(cache: &CacheStore, token: Option<String>, lang: Lang) -> Result<()> {
    let id_token = match token {
        Some(t) => t,
        None => {
            println!("Google OAuth Login");
            println!("==================");
            println!();
            println!("To login with Google, you need to obtain an ID token.");
            println!("Client ID: {}", ShkoloClient::google_client_id());
            println!();
            println!("Steps:");
            println!("1. Use Google OAuth to authenticate with the client ID above");
            println!("2. Copy the ID token from the response");
            println!("3. Run: shkolo login-google --token <YOUR_ID_TOKEN>");
            println!();
            println!("Or paste the ID token now:");
            print!("ID Token: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if id_token.is_empty() {
        return Err(anyhow!("No token provided"));
    }

    let mut client = ShkoloClient::new();
    let response = client.login_google(&id_token).await?;

    // Save token
    let user_data = serde_json::to_value(&response)?;
    cache.save_token(
        &client.token().unwrap(),
        client.school_year(),
        selected_user_id(&response),
        Some(user_data),
    )?;

    println!("{}", T::cli_logged_in_google(lang));

    Ok(())
}

async fn login_web(cache: &CacheStore, cookie: &str, xsrf_token: Option<&str>, lang: Lang) -> Result<()> {
    let client = ShkoloClient::with_cookie(
        cookie.to_string(),
        xsrf_token.map(|s| s.to_string()),
        None,
    );

    // Verify cookie works by probing the API
    match client.verify_auth().await {
        Ok(_) => {
            // Try to get school year and user (school) info
            let mut school_year = None;
            let mut user_id = None;
            let mut user_data = None;
            if let Ok(users_response) = client.get_users_and_years().await {
                if let Some(ref entries) = users_response.users {
                    // Prefer the isSelected entry
                    if let Some(selected) = entries.iter().find(|e| e.is_selected == Some(true)) {
                        school_year = selected.school_year;
                        user_id = selected.id;
                    } else {
                        school_year = entries.iter().filter_map(|e| e.school_year).max();
                    }
                }
                user_data = serde_json::to_value(&users_response).ok();
            }

            let token_data = cache::TokenData {
                token: cookie.to_string(),
                school_year,
                user_data,
                auth_type: Some("cookie".to_string()),
                xsrf_token: xsrf_token.map(|s| s.to_string()),
                user_id,
            };
            cache.save_token_full(&token_data)?;

            println!("{}", T::cli_logged_in_web(lang));
            Ok(())
        }
        Err(_) => {
            eprintln!("{}", T::cli_web_login_failed(lang));
            Err(anyhow!("Web authentication failed"))
        }
    }
}

async fn logout(cache: &CacheStore, lang: Lang) -> Result<()> {
    if let Ok(token_data) = cache.load_token() {
        let mut client = ShkoloClient::with_token(token_data.token, token_data.school_year);
        let _ = client.logout().await;
    }

    cache.clear_token()?;
    println!("{}", T::cli_logged_out(lang));

    Ok(())
}

fn show_status(cache: &CacheStore, lang: Lang) -> Result<()> {
    match cache.load_token() {
        Ok(token_data) => {
            println!("{}", T::cli_status_authenticated(lang));

            if let Some(ref user_data) = token_data.user_data {
                if let Some(name) = user_data.get("names").and_then(|v| v.as_str()) {
                    println!("{}: {}", T::cli_user(lang), name);
                }
                if let Some(users) = user_data.get("users").and_then(|v| v.as_array()) {
                    for user in users {
                        if let Some(name) = user.get("names").and_then(|v| v.as_str()) {
                            println!("{}: {}", T::cli_user(lang), name);
                        }
                    }
                }
            }

            if let Some(ref auth_type) = token_data.auth_type {
                println!("{}: {}", T::cli_auth_type(lang), auth_type);
            }

            if let Some(year) = token_data.school_year {
                println!("{}: {} ({})", T::cli_school_year_id(lang), year, SchoolContext::year_label_for(year));
            }

            // Active school and the other schools selectable this year
            let contexts = contexts_from_token(&token_data);
            let active = token_data.user_id.zip(token_data.school_year)
                .or_else(|| contexts.iter().find(|c| c.is_selected).map(|c| c.key()));
            if let Some(ctx) = contexts.iter().find(|c| Some(c.key()) == active) {
                println!("{}: {}", T::cli_school(lang), ctx.school_name);
            }
            let current = SchoolContext::current_year(&contexts);
            let current_schools: Vec<&SchoolContext> = contexts.iter().filter(|c| c.is_current(current)).collect();
            if current_schools.len() > 1 {
                println!("{}:", T::cli_available_schools(lang));
                for c in current_schools {
                    println!("  {} {} ({}) [user {}]", if Some(c.key()) == active { "*" } else { " " }, c.school_name, c.year_label(), c.user_id);
                }
            }

            println!();
            println!("{}: {}", T::cli_cache_dir(lang), cache.cache_dir().display());
            println!("{}: {} {}", T::cli_cache_ttl(lang), cache.ttl(), T::cli_seconds(lang));
        }
        Err(_) => {
            println!("{}", T::cli_status_not_authenticated(lang));
            println!();
            println!("{}", T::cli_run_login_hint(lang));
        }
    }

    Ok(())
}

fn set_lang(cache: &CacheStore, language: &str) -> Result<()> {
    match Lang::from_str(language) {
        Some(lang) => {
            let mut ui_config = cache.load_ui_config();
            ui_config.language = Some(lang.as_str().to_string());
            cache.save_ui_config(&ui_config)?;
            println!("{} {}", T::cli_lang_set(lang), lang.as_str());
            Ok(())
        }
        None => {
            println!("{}", T::cli_lang_invalid(Lang::En));
            Err(anyhow!("Invalid language: {}", language))
        }
    }
}

async fn cache_command(cache: &CacheStore, clear: bool, clear_all: bool, refresh: bool, lang: Lang) -> Result<()> {
    if clear_all {
        cache.clear_all()?;
        println!("{}", T::cli_cache_cleared_all(lang));
    } else if clear {
        cache.clear()?;
        println!("{}", T::cli_cache_cleared(lang));
    }

    if refresh {
        let client = get_authenticated_client(cache)?;

        println!("{}", T::cli_refreshing_all(lang));

        // Get students
        let pupils_response = client.get_pupils().await?;
        let mut students = Vec::new();

        if let Some(child_pupils) = pupils_response.child_pupils {
            for (id, pupil) in child_pupils {
                students.push(Student::from_child_pupil(&id, &pupil));
            }
        }

        cache.save_students(&students)?;
        println!("  {} {}", T::cli_refreshed_students(lang), students.len());

        let today = get_today_date();

        for student in &students {
            // Refresh homework
            if let Ok(courses_response) = client.get_homework_courses(student.id).await {
                let mut homework = Vec::new();
                if let Some(courses) = courses_response.courses {
                    let counts = courses_response.cyc_group_homeworks_count.unwrap_or_default();
                    for course in courses {
                        if let Some(cyc_group_id) = course.cyc_group_id {
                            if counts.get(&cyc_group_id.to_string()).copied().unwrap_or(0) == 0 {
                                continue;
                            }
                            let base = course.course_name.or(course.course_short_name).unwrap_or_default();
                            let subject = match course.course_type_name {
                                Some(ref t) if !t.is_empty() => format!("{} ({})", base, t),
                                _ => base,
                            };
                            if let Ok(hw_response) = client.get_homework_list(cyc_group_id).await {
                                if let Some(items) = hw_response.homeworks {
                                    for item in items {
                                        homework.push(Homework::from_item(&item, &subject));
                                    }
                                }
                            }
                        }
                    }
                }
                homework.sort_by(|a, b| b.date_sort.cmp(&a.date_sort));
                cache.save_homework(student.id, &homework)?;
            }

            // Refresh grades (try detailed endpoint first)
            let mut grades_saved = false;
            if let Ok(response) = client.get_grades(student.id).await {
                let raw = response.grades.unwrap_or_default();
                if !raw.is_empty() {
                    let grades: Vec<_> = Grade::from_grades_list(&raw)
                        .into_iter()
                        .filter(|g| g.has_grades())
                        .collect();
                    cache.save_grades(student.id, &grades)?;
                    grades_saved = true;
                }
            }
            if !grades_saved {
                if let Ok(grades_response) = client.get_grades_summary(student.id).await {
                    let courses = grades_response.grades.or(grades_response.courses).unwrap_or_default();
                    let grades: Vec<_> = courses.iter()
                        .map(Grade::from_course_grades)
                        .filter(|g| g.has_grades())
                        .collect();
                    cache.save_grades(student.id, &grades)?;
                }
            }

            // Refresh schedule
            if let Ok(schedule_response) = client.get_pupil_schedule(student.id, &today).await {
                let hours = schedule_response.schedule_hours.or(schedule_response.data).unwrap_or_default();
                let mut schedule: Vec<_> = hours.iter().map(ScheduleHour::from_raw).collect();
                schedule.sort_by_key(|h| h.hour_number);
                cache.save_schedule(student.id, &today, &schedule)?;
            }

            println!("  {} {}", T::cli_refreshed_data_for(lang), student.name);
        }

        println!("{}", T::cli_all_data_refreshed(lang));
    }

    if !clear && !clear_all && !refresh {
        println!("{}: {}", T::cli_cache_dir(lang), cache.cache_dir().display());
        println!("{}: {} {}", T::cli_cache_ttl(lang), cache.ttl(), T::cli_seconds(lang));
        println!();
        println!("Options:");
        println!("  --clear     Clear cache (preserves token)");
        println!("  --clear-all Clear all cache including token");
        println!("  --refresh   Force refresh all data");
    }

    Ok(())
}

fn get_authenticated_client(cache: &CacheStore) -> Result<ShkoloClient> {
    let token_data = cache.load_token()
        .map_err(|_| anyhow!("Not authenticated. Run 'shkolo login' or 'shkolo import-token' first."))?;

    let is_cookie = token_data.auth_type.as_deref() == Some("cookie");
    let client = if is_cookie {
        ShkoloClient::with_cookie(
            token_data.token,
            token_data.xsrf_token,
            token_data.school_year,
        )
    } else {
        ShkoloClient::with_token(token_data.token, token_data.school_year)
    };
    client.set_user_id(token_data.user_id);
    Ok(client)
}

async fn get_students(
    client: &ShkoloClient,
    cache: &CacheStore,
    force_refresh: bool,
) -> Result<(Vec<Student>, bool, Option<String>)> {
    // Check cache first. A list without school tags can only be trusted for
    // single-school accounts; otherwise it came from a degraded fetch.
    if !force_refresh {
        if let Some((students, age, expired)) = cache.get_students() {
            let untagged = students.iter().any(|s| s.context().is_none());
            let incomplete = untagged && known_school_count(cache) > 1;
            if !expired && !incomplete {
                return Ok((students, true, Some(age)));
            }
        }
    }

    // Fetch from API: the pupils endpoint only returns the selected school's children,
    // so visit every school of the current year and tag the pupils with their school
    let contexts = current_contexts(client, cache).await?;
    let mut students = Vec::new();
    if contexts.is_empty() {
        students.extend(fetch_pupils(client).await?);
    } else {
        for ctx in &contexts {
            ensure_context_persisted(client, cache, ctx.user_id, ctx.school_year).await?;
            students.extend(fetch_pupils(client).await?.into_iter().map(|s| s.with_context(ctx)));
        }
    }

    // Group by school, then by name (stable order for the sidebar)
    students.sort_by(|a, b| a.school_name.cmp(&b.school_name).then_with(|| a.name.cmp(&b.name)));
    cache.save_students(&students)?;

    Ok((students, false, Some("just now".to_string())))
}

/// Children of the school the server is currently on
async fn fetch_pupils(client: &ShkoloClient) -> Result<Vec<Student>> {
    let pupils_response = client.get_pupils().await?;
    let mut students = Vec::new();
    if let Some(child_pupils) = pupils_response.child_pupils {
        for (id, pupil) in child_pupils {
            students.push(Student::from_child_pupil(&id, &pupil));
        }
    }
    Ok(students)
}

async fn get_homework(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    force_refresh: bool,
) -> Result<(Vec<Homework>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((homework, age, expired)) = cache.get_homework(student_id) {
            if !expired {
                return Ok((homework, true, Some(age)));
            }
        }
    }

    // Fetch from API
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

                let base = course.course_name
                    .or(course.course_short_name)
                    .unwrap_or_else(|| "Unknown".to_string());
                let subject = match &course.course_type_name {
                    Some(t) if !t.is_empty() => format!("{} ({})", base, t),
                    _ => base,
                };

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

    all_homework.sort_by(|a, b| b.date_sort.cmp(&a.date_sort));
    cache.save_homework(student_id, &all_homework)?;

    Ok((all_homework, false, Some("just now".to_string())))
}

async fn get_grades(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    force_refresh: bool,
) -> Result<(Vec<Grade>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((grades, age, expired)) = cache.get_grades(student_id) {
            if !expired {
                return Ok((grades, true, Some(age)));
            }
        }
    }

    // Try detailed /grades endpoint first (has dates, comments, types)
    if let Ok(response) = client.get_grades(student_id).await {
        let raw = response.grades.unwrap_or_default();
        if !raw.is_empty() {
            let grades: Vec<Grade> = Grade::from_grades_list(&raw)
                .into_iter()
                .filter(|g| g.has_grades())
                .collect();
            if !grades.is_empty() {
                cache.save_grades(student_id, &grades)?;
                return Ok((grades, false, Some("just now".to_string())));
            }
        }
    }

    // Fallback to summary endpoint (no dates/comments)
    let response = client.get_grades_summary(student_id).await?;
    let courses = response.grades.or(response.courses).unwrap_or_default();
    let grades: Vec<Grade> = courses
        .iter()
        .map(Grade::from_course_grades)
        .filter(|g| g.has_grades())
        .collect();

    cache.save_grades(student_id, &grades)?;

    Ok((grades, false, Some("just now".to_string())))
}

async fn get_schedule(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    date: &str,
    force_refresh: bool,
) -> Result<(Vec<ScheduleHour>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((schedule, age, expired)) = cache.get_schedule(student_id, date) {
            if !expired {
                return Ok((schedule, true, Some(age)));
            }
        }
    }

    // Fetch from API
    let response = client.get_pupil_schedule(student_id, date).await?;

    let hours = response.schedule_hours.or(response.data).unwrap_or_default();
    let mut schedule: Vec<ScheduleHour> = hours.iter().map(ScheduleHour::from_raw).collect();
    schedule.sort_by_key(|h| h.hour_number);

    cache.save_schedule(student_id, date, &schedule)?;

    Ok((schedule, false, Some("just now".to_string())))
}

async fn get_absences(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    force_refresh: bool,
) -> Result<(Vec<Absence>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((absences, age, expired)) = cache.get_absences(student_id) {
            if !expired {
                return Ok((absences, true, Some(age)));
            }
        }
    }

    // Fetch from API
    let response = client.get_absences(student_id).await?;

    let mut absences: Vec<Absence> = response.absences
        .unwrap_or_default()
        .iter()
        .map(Absence::from_raw)
        .collect();

    // Sort by date (newest first)
    absences.sort_by(|a, b| {
        b.date_sort.cmp(&a.date_sort)
            .then_with(|| a.hour.cmp(&b.hour))
    });

    cache.save_absences(student_id, &absences)?;

    Ok((absences, false, Some("just now".to_string())))
}

async fn get_feedbacks(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    force_refresh: bool,
) -> Result<(Vec<Feedback>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((feedbacks, age, expired)) = cache.get_feedbacks(student_id) {
            if !expired {
                return Ok((feedbacks, true, Some(age)));
            }
        }
    }

    // Fetch from API
    let response = client.get_feedbacks(student_id).await?;

    let mut feedbacks: Vec<Feedback> = response.data
        .or(response.feedbacks)
        .unwrap_or_default()
        .iter()
        .map(Feedback::from_raw)
        .collect();

    // Sort by date (newest first)
    feedbacks.sort_by(Feedback::cmp_by_date);

    cache.save_feedbacks(student_id, &feedbacks)?;

    Ok((feedbacks, false, Some("just now".to_string())))
}

async fn get_notifications(
    client: &ShkoloClient,
    cache: &CacheStore,
    force_refresh: bool,
) -> Result<(Vec<Notification>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((notifications, age, expired)) = cache.get_notifications() {
            if !expired {
                return Ok((notifications, true, Some(age)));
            }
        }
    }

    // Fetch from every current-year school (notifications are per parent user)
    let contexts = current_contexts(client, cache).await?;
    let mut notifications = Vec::new();
    if contexts.is_empty() {
        notifications = fetch_notifications(client).await?;
    } else {
        for ctx in &contexts {
            ensure_context_persisted(client, cache, ctx.user_id, ctx.school_year).await?;
            for mut n in fetch_notifications(client).await? {
                n.school_name = Some(ctx.school_name.clone());
                n.user_id = Some(ctx.user_id);
                n.school_year = Some(ctx.school_year);
                notifications.push(n);
            }
        }
    }
    // Newest first; tie-break on id for a stable order
    notifications.sort_by(|a, b| {
        date_key(&b.date).cmp(&date_key(&a.date)).then_with(|| b.id.cmp(&a.id))
    });

    cache.save_notifications(&notifications)?;

    Ok((notifications, false, Some("just now".to_string())))
}

/// Notifications of the school the server is currently on
async fn fetch_notifications(client: &ShkoloClient) -> Result<Vec<Notification>> {
    let response = client.get_notifications(1).await?;
    Ok(response.data
        .or(response.notifications)
        .unwrap_or_default()
        .iter()
        .map(Notification::from_raw)
        .collect())
}

/// Message threads from every current-year school (the messenger is per parent user)
async fn get_messages(
    client: &ShkoloClient,
    cache: &CacheStore,
    force_refresh: bool,
) -> Result<(Vec<MessageThread>, bool, Option<String>)> {
    if !force_refresh {
        if let Some((messages, age, expired)) = cache.get_messages() {
            if !expired {
                return Ok((messages, true, Some(age)));
            }
        }
    }

    let contexts = current_contexts(client, cache).await?;
    let mut messages = Vec::new();
    if contexts.is_empty() {
        messages = fetch_messages(client).await?;
    } else {
        for ctx in &contexts {
            ensure_context_persisted(client, cache, ctx.user_id, ctx.school_year).await?;
            for mut m in fetch_messages(client).await? {
                m.school_name = Some(ctx.school_name.clone());
                m.user_id = Some(ctx.user_id);
                m.school_year = Some(ctx.school_year);
                messages.push(m);
            }
        }
    }
    messages.sort_by(|a, b| {
        date_key(&b.updated_at).cmp(&date_key(&a.updated_at)).then_with(|| b.id.cmp(&a.id))
    });

    cache.save_messages(&messages)?;

    Ok((messages, false, Some("just now".to_string())))
}

/// Message threads of the school the server is currently on
async fn fetch_messages(client: &ShkoloClient) -> Result<Vec<MessageThread>> {
    let raw_threads = client.get_messenger_threads(None).await?;
    Ok(raw_threads.iter().map(MessageThread::from_raw).collect())
}

/// Sort key for API dates: "DD.MM.YYYY[ HH:MM]" becomes "YYYY-MM-DD[ HH:MM]",
/// anything else (ISO already) is used as is
fn date_key(date: &str) -> String {
    let (day_part, rest) = match date.split_once(' ') {
        Some((d, r)) => (d, format!(" {}", r)),
        None => (date, String::new()),
    };
    let parts: Vec<&str> = day_part.split('.').collect();
    if parts.len() == 3 && parts[2].len() == 4 {
        format!("{}-{}-{}{}", parts[2], parts[1], parts[0], rest)
    } else {
        date.to_string()
    }
}

fn select_students<'a>(students: &'a [Student], selector: Option<&str>) -> Vec<&'a Student> {
    match selector {
        None => students.iter().collect(),
        Some(s) => {
            // Try parsing as index first
            if let Ok(idx) = s.parse::<usize>() {
                if idx > 0 && idx <= students.len() {
                    return vec![&students[idx - 1]];
                }
            }

            // Try matching by name (case-insensitive, partial match)
            let lower = s.to_lowercase();
            let matches: Vec<_> = students
                .iter()
                .filter(|student| student.name.to_lowercase().contains(&lower))
                .collect();

            if matches.is_empty() {
                students.iter().collect()
            } else {
                matches
            }
        }
    }
}

fn output_json<T: serde::Serialize>(data: &T, format: &str) -> Result<()> {
    let output = if format == "compact" {
        serde_json::to_string(data)?
    } else {
        serde_json::to_string_pretty(data)?
    };

    println!("{}", output);
    Ok(())
}

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

fn get_today_date() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}

/// Refresh all data in the background and return the result
async fn refresh_data_background(
    client: &ShkoloClient,
    cache: &CacheStore,
    force_refresh: bool,
    _student_ids: Vec<i64>,
) -> Result<BackgroundResult> {
    // Where the server really is before we start hopping between schools (the phone
    // app shares the token and may have moved it since we last looked)
    let home_context = client.sync_context().await.ok().flatten().or_else(|| client.current_context());

    // Fetch students of all current-year schools
    let (students, _, _) = get_students(client, cache, force_refresh).await?;

    // Fetch data for each student (per-pupil endpoints need the pupil's school selected)
    let mut student_data_list = Vec::new();
    let today = get_today_date();

    for student in students {
        ensure_student_context(client, cache, &student).await?;

        // A single failed call (the API returns sporadic 502s) must not hide the
        // whole refresh: fall back to whatever is cached for that item.
        // The school is re-selected before every call: a foreground action (opening
        // a message thread) can move the server to another school between two of them.
        let (homework, _, hw_age) = match get_homework(client, cache, student.id, force_refresh).await {
            Ok(r) => r,
            Err(_) => cached_or_empty(cache.get_homework(student.id)),
        };
        ensure_student_context(client, cache, &student).await?;
        let (grades, _, grades_age) = match get_grades(client, cache, student.id, force_refresh).await {
            Ok(r) => r,
            Err(_) => cached_or_empty(cache.get_grades(student.id)),
        };
        ensure_student_context(client, cache, &student).await?;
        let (absences, _, absences_age) = match get_absences(client, cache, student.id, force_refresh).await {
            Ok(r) => r,
            Err(_) => cached_or_empty(cache.get_absences(student.id)),
        };
        ensure_student_context(client, cache, &student).await?;
        let (feedbacks, _, feedbacks_age) = match get_feedbacks(client, cache, student.id, force_refresh).await {
            Ok(r) => r,
            Err(_) => cached_or_empty(cache.get_feedbacks(student.id)),
        };

        // Get schedule - use today for background refresh
        ensure_student_context(client, cache, &student).await?;
        let (schedule, _, schedule_age) = match get_schedule(client, cache, student.id, &today, force_refresh).await {
            Ok(r) => r,
            Err(_) => cached_or_empty(cache.get_schedule(student.id, &today)),
        };

        // Events (upcoming tests etc.); cached copy if the API call fails
        ensure_student_context(client, cache, &student).await?;
        let events = match client.get_pupil_events(student.id).await {
            Ok(response) => {
                let events: Vec<models::Event> = response.invitations
                    .unwrap_or_default()
                    .iter()
                    .map(models::Event::from_raw)
                    .collect();
                let _ = cache.save_events(student.id, &events);
                events
            }
            Err(_) => cache.get_events(student.id).map(|(e, _, _)| e).unwrap_or_default(),
        };

        student_data_list.push(StudentData {
            student,
            homework,
            grades,
            schedule,
            events,
            absences,
            feedbacks,
            homework_age: hw_age,
            grades_age,
            schedule_age,
            absences_age,
            feedbacks_age,
        });
    }

    // Compute effective date (check if school day is over or weekend)
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let schedules: Vec<&[ScheduleHour]> = student_data_list.iter().map(|s| s.schedule.as_slice()).collect();
    let effective_date = tui::app::compute_effective_date(
        &today,
        now.hour(),
        now.minute(),
        &schedules,
    );

    // If effective date differs from today, fetch next school day's schedule for all students
    if effective_date != today {
        for data in &mut student_data_list {
            ensure_student_context(client, cache, &data.student).await?;
            if let Ok((schedule, _, schedule_age)) = get_schedule(client, cache, data.student.id, &effective_date, force_refresh).await {
                data.schedule = schedule;
                data.schedule_age = schedule_age;
            }
        }
    }

    // Notifications and messages from all schools
    let (notifications, _, _) = get_notifications(client, cache, force_refresh).await?;
    let messages = get_messages(client, cache, force_refresh).await.map(|(m, _, _)| m).unwrap_or_default();

    // Refresh the list of school contexts (non-fatal if it fails) and keep it in the token file
    let contexts = client.get_users_and_years().await.ok().and_then(|r| r.users);
    if let Some(ref entries) = contexts {
        if let Ok(mut token_data) = cache.load_token() {
            if let Ok(json) = serde_json::to_value(entries) {
                store_users_in_token(&mut token_data, json);
                let _ = cache.save_token_full(&token_data);
            }
        }
    }

    // Leave the server where it was (this also keeps the mobile app on its school)
    if let Some((user_id, school_year)) = home_context {
        let _ = ensure_context_persisted(client, cache, user_id, school_year).await;
    }

    Ok(BackgroundResult::DataRefresh {
        students: student_data_list,
        notifications,
        messages,
        effective_date,
        contexts,
    })
}

/// Cached data (even if expired) in the shape the get_* helpers return, or nothing
fn cached_or_empty<T>(cached: Option<(Vec<T>, String, bool)>) -> (Vec<T>, bool, Option<String>) {
    match cached {
        Some((data, age, _)) => (data, true, Some(age)),
        None => (Vec::new(), false, None),
    }
}

/// Refresh schedule for a specific student/date in the background
async fn refresh_schedule_background(
    client: &ShkoloClient,
    cache: &CacheStore,
    student_id: i64,
    date: &str,
) -> Result<BackgroundResult> {
    let home_context = client.current_context();
    if let Some((students, _, _)) = cache.get_students() {
        if let Some(student) = students.iter().find(|s| s.id == student_id) {
            ensure_student_context(client, cache, student).await?;
        }
    }
    let result = get_schedule(client, cache, student_id, date, true).await;
    if let Some((user_id, school_year)) = home_context {
        let _ = ensure_context_persisted(client, cache, user_id, school_year).await;
    }
    let (schedule, _, _) = result?;

    Ok(BackgroundResult::ScheduleRefresh {
        student_id,
        date: date.to_string(),
        schedule,
    })
}

// --- School context helpers ---

/// School contexts of the current school year. Read from the token file; fetched
/// live (and stored) when the file has none yet. Fails when the list cannot be
/// determined at all, so callers never silently degrade to a single school.
async fn current_contexts(client: &ShkoloClient, cache: &CacheStore) -> Result<Vec<SchoolContext>> {
    let mut contexts = cache.load_token().map(|t| contexts_from_token(&t)).unwrap_or_default();
    if contexts.is_empty() {
        let response = client.get_users_and_years().await
            .map_err(|e| anyhow!("Could not determine the account's schools: {}", e))?;
        let entries = response.users.unwrap_or_default();
        if let Ok(mut token_data) = cache.load_token() {
            if let Ok(json) = serde_json::to_value(&entries) {
                store_users_in_token(&mut token_data, json);
                let _ = cache.save_token_full(&token_data);
            }
        }
        contexts = SchoolContext::from_entries(&entries);
    }
    let current = SchoolContext::current_year(&contexts);
    contexts.retain(|c| c.is_current(current));
    Ok(contexts)
}

/// Number of schools in the current year according to the token file (0 if unknown)
fn known_school_count(cache: &CacheStore) -> usize {
    let contexts = cache.load_token().map(|t| contexts_from_token(&t)).unwrap_or_default();
    let current = SchoolContext::current_year(&contexts);
    contexts.iter().filter(|c| c.is_current(current)).count()
}

/// Make sure the server is on the given school context and keep the token file
/// in step with where the client now knows the server to be.
async fn ensure_context_persisted(client: &ShkoloClient, cache: &CacheStore, user_id: i64, school_year: i64) -> Result<bool> {
    let switched = client.ensure_context(user_id, school_year).await?;
    persist_context(client, cache);
    Ok(switched)
}

/// Switch to the given school context unconditionally, even if the client believes
/// the server is already there (used after the server proved that belief wrong).
async fn switch_context_persisted(client: &ShkoloClient, cache: &CacheStore, user_id: i64, school_year: i64) -> Result<()> {
    client.switch_user_and_year(user_id, school_year, None).await?;
    persist_context(client, cache);
    Ok(())
}

/// Record the client's current token and school context in the token file if they changed
fn persist_context(client: &ShkoloClient, cache: &CacheStore) {
    let Ok(mut token_data) = cache.load_token() else { return };
    let current = client.current_context();
    let stored = match (token_data.user_id, token_data.school_year) {
        (Some(u), Some(y)) => Some((u, y)),
        _ => None,
    };
    let token = client.token();
    if current == stored && (token.is_none() || token.as_deref() == Some(token_data.token.as_str())) {
        return;
    }
    if let Some(token) = token {
        token_data.token = token;
    }
    if let Some((user_id, school_year)) = current {
        token_data.user_id = Some(user_id);
        token_data.school_year = Some(school_year);
    }
    let _ = cache.save_token_full(&token_data);
}

/// Select the school a pupil belongs to before calling per-pupil endpoints
async fn ensure_student_context(client: &ShkoloClient, cache: &CacheStore, student: &Student) -> Result<()> {
    if let Some((user_id, school_year)) = student.context() {
        ensure_context_persisted(client, cache, user_id, school_year).await?;
    }
    Ok(())
}

/// The school context a message thread was listed under, if known
fn thread_context(threads: &[MessageThread], thread_id: i64) -> Option<(i64, i64)> {
    threads.iter()
        .find(|m| m.id == thread_id)
        .and_then(|m| Some((m.user_id?, m.school_year?)))
}

/// Run a messenger call for a thread with that thread's school selected on the server.
///
/// Threads belong to a parent user, i.e. to one school, and the server answers 403
/// for a thread of a school that is not selected. The selection is per token, so the
/// client's idea of it can be stale (the phone app or a refresh in flight may have
/// moved it). On 403 the switch is therefore repeated explicitly and the call retried;
/// a thread of unknown school is tried under every current-year school.
/// Returns the result together with the school context that served it.
async fn with_thread_context<T, F, Fut>(
    client: &ShkoloClient,
    cache: &CacheStore,
    context: Option<(i64, i64)>,
    op: F,
) -> Result<(T, Option<(i64, i64)>)>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut switch_error = None;
    if let Some((user_id, school_year)) = context {
        if let Err(e) = ensure_context_persisted(client, cache, user_id, school_year).await {
            switch_error = Some(e);
        }
    }

    let first_error = match op().await {
        Ok(value) => return Ok((value, context.or_else(|| client.current_context()))),
        Err(e) if api::client::is_forbidden(&e) => e,
        Err(e) => return Err(e),
    };

    // The server is on another school than believed: switch explicitly and retry
    let candidates: Vec<(i64, i64)> = match context {
        Some(ctx) => vec![ctx],
        None => {
            let server = client.sync_context().await.ok().flatten();
            current_contexts(client, cache).await.unwrap_or_default()
                .into_iter()
                .map(|c| (c.user_id, c.school_year))
                .filter(|c| Some(*c) != server)
                .collect()
        }
    };
    let mut last_error = switch_error.unwrap_or(first_error);
    for (user_id, school_year) in candidates {
        if let Err(e) = switch_context_persisted(client, cache, user_id, school_year).await {
            last_error = e;
            continue;
        }
        match op().await {
            Ok(value) => return Ok((value, Some((user_id, school_year)))),
            Err(e) if api::client::is_forbidden(&e) => last_error = e,
            Err(e) => return Err(e),
        }
    }
    Err(last_error)
}

/// Load a thread's messages in the TUI, selecting the thread's school on the server
async fn load_thread_messages(client: &ShkoloClient, cache: &CacheStore, app: &mut App, thread_id: i64) -> Result<Vec<Message>> {
    let context = thread_context(&app.messages, thread_id);
    let (messages, served_by) = with_thread_context(client, cache, context, || client.get_thread_messages(thread_id)).await?;
    note_thread_context(app, thread_id, served_by);
    Ok(messages)
}

/// Reply to a thread in the TUI, selecting the thread's school on the server
async fn send_thread_reply(client: &ShkoloClient, cache: &CacheStore, app: &mut App, thread_id: i64, message: &str) -> Result<()> {
    let context = thread_context(&app.messages, thread_id);
    let (_, served_by) = with_thread_context(client, cache, context, || client.reply_to_thread(thread_id, message)).await?;
    note_thread_context(app, thread_id, served_by);
    Ok(())
}

/// Remember which school served a thread: the sidebar selection follows it, and a
/// thread listed without a school (older cache) gets tagged for next time
fn note_thread_context(app: &mut App, thread_id: i64, context: Option<(i64, i64)>) {
    let Some((user_id, school_year)) = context else { return };
    app.set_active_context(user_id, school_year);
    if let Some(thread) = app.messages.iter_mut().find(|m| m.id == thread_id) {
        if thread.user_id.is_none() {
            thread.user_id = Some(user_id);
            thread.school_year = Some(school_year);
        }
    }
}

/// The parent user id the server marks as selected after login
fn selected_user_id(response: &UsersAndYearsResponse) -> Option<i64> {
    response.users.iter().flatten()
        .find(|e| e.is_selected == Some(true))
        .and_then(|e| e.id)
}

/// School contexts stored in the token file's user data (from login or a later refresh)
fn contexts_from_token(token_data: &cache::TokenData) -> Vec<SchoolContext> {
    token_data.user_data.as_ref()
        .and_then(|d| d.get("users"))
        .and_then(|u| serde_json::from_value::<Vec<UserYearEntry>>(u.clone()).ok())
        .map(|entries| SchoolContext::from_entries(&entries))
        .unwrap_or_default()
}

/// Store a fresh usersAndYears `users` array in the token's user data, keeping other keys
fn store_users_in_token(token_data: &mut cache::TokenData, users_json: serde_json::Value) {
    // Accept either the whole response ({"users": [...]}) or the bare array
    let users = users_json.get("users").cloned().unwrap_or(users_json);
    match token_data.user_data.as_mut() {
        Some(serde_json::Value::Object(map)) => {
            map.insert("users".to_string(), users);
        }
        _ => token_data.user_data = Some(serde_json::json!({ "users": users })),
    }
}

/// Persist a completed server-side context switch: token (possibly new), year, user, contexts
fn persist_context_switch(
    cache: &CacheStore,
    client: &ShkoloClient,
    users: &UsersAndYearsResponse,
    user_id: i64,
    school_year: i64,
) {
    if let Ok(mut token_data) = cache.load_token() {
        if let Some(token) = client.token() {
            token_data.token = token;
        }
        token_data.school_year = Some(school_year);
        token_data.user_id = Some(user_id);
        if let Ok(json) = serde_json::to_value(users) {
            store_users_in_token(&mut token_data, json);
        }
        let _ = cache.save_token_full(&token_data);
    }
}
