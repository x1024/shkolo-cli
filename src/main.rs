mod api;
mod cache;
mod i18n;
mod models;
mod tui;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, MouseEvent, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use time::OffsetDateTime;

use api::ShkoloClient;
use cache::CacheStore;
use models::*;
use tui::{App, draw, handle_key, handlers::Action};

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

    /// Logout and clear token
    Logout,

    /// Show authentication status
    Status,

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

    /// Get absences (testing endpoint)
    Absences {
        /// Student name or index (optional, defaults to first)
        student: Option<String>,
    },

    /// Try messages endpoints (discovery)
    Messages,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get cache TTL from env, config, or default
    let ttl = cli.cache_ttl
        .or_else(|| std::env::var("SHKOLO_CACHE_TTL").ok().and_then(|v| v.parse().ok()));

    let cache = CacheStore::new(ttl)?;

    match cli.command {
        Commands::Json { command, format } => {
            run_json_command(command, &cache, cli.refresh, cli.no_cache, &format).await
        }
        Commands::Tui => run_tui(&cache).await,
        Commands::ImportToken => import_token(&cache),
        Commands::Login { username, password } => login(&cache, username, password).await,
        Commands::LoginGoogle { token } => login_google(&cache, token).await,
        Commands::Logout => logout(&cache).await,
        Commands::Status => show_status(&cache),
        Commands::Cache { clear, clear_all, refresh } => {
            cache_command(&cache, clear, clear_all, refresh).await
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

            if let Some(s) = selected.first() {
                let response = client.get_absences(s.id).await?;
                output_json(&api::ApiResponse::new(response, false, None), format)?;
            } else {
                output_json(&api::ApiResponse::new(serde_json::json!({"error": "No student found"}), false, None), format)?;
            }
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
    }

    Ok(())
}

async fn run_tui(cache: &CacheStore) -> Result<()> {
    let mut client = get_authenticated_client(cache)?;

    // Setup terminal with mouse support
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Load user name from token cache
    if let Ok(token_data) = cache.load_token() {
        if let Some(data) = token_data.user_data {
            // Try "names" field first (from import)
            if let Some(names) = data.get("names").and_then(|v: &serde_json::Value| v.as_str()) {
                app.user_name = Some(names.to_string());
            }
            // Try "users" array (from login)
            else if let Some(users) = data.get("users").and_then(|v: &serde_json::Value| v.as_array()) {
                if let Some(first) = users.first() {
                    if let Some(names) = first.get("names").and_then(|v: &serde_json::Value| v.as_str()) {
                        app.user_name = Some(names.to_string());
                    }
                }
            }
        }
    }

    // Load UI configuration (pane sizes, etc.)
    let ui_config = cache.load_ui_config();
    if let Some(width) = ui_config.students_pane_width {
        app.students_pane_width = width;
    }

    // Load cached data first
    app.load_from_cache(cache).await;

    // If no cached data, refresh
    if app.students.is_empty() {
        // Show loading state
        app.loading = true;
        app.set_status("Loading data...");
        terminal.draw(|f| draw(f, &app))?;

        if let Err(e) = app.refresh_data(&client, cache, false).await {
            app.set_status(format!("Error: {}", e));
        }
    }

    // Main loop - resource-efficient event handling
    let mut last_time_update = std::time::Instant::now();

    loop {
        // Update time periodically for schedule highlighting (once per minute is enough)
        if last_time_update.elapsed() >= Duration::from_secs(60) {
            app.update_time();
            last_time_update = std::time::Instant::now();
        }

        // Tick for loading animation
        if app.loading {
            app.tick();
        }

        terminal.draw(|f| draw(f, &app))?;

        // Determine poll timeout based on loading state
        let poll_timeout = if app.loading {
            // Fast polling during loading for spinner animation
            Duration::from_millis(100)
        } else {
            // Block for up to 60 seconds when idle - minimal CPU usage
            Duration::from_secs(60)
        };

        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) => match handle_key(&mut app, key) {
                    Action::Refresh => {
                        // Show loading state before starting refresh
                        app.loading = true;
                        app.set_status("Refreshing...");
                        terminal.draw(|f| draw(f, &app))?;

                        if let Err(e) = app.refresh_data(&client, cache, false).await {
                            app.set_status(format!("Error: {}", e));
                        }
                    }
                    Action::RefreshAll => {
                        // Show loading state before starting refresh
                        app.loading = true;
                        app.set_status("Refreshing all...");
                        terminal.draw(|f| draw(f, &app))?;

                        if let Err(e) = app.refresh_data(&client, cache, true).await {
                            app.set_status(format!("Error: {}", e));
                        }
                    }
                    Action::Logout => {
                        // Clear token and exit
                        if let Err(e) = cache.clear_token() {
                            app.set_status(format!("Logout error: {}", e));
                        } else {
                            app.set_status("Logged out. Restart to log in again.");
                            app.user_name = None;
                            // Exit after logout
                            app.quit();
                        }
                    }
                    Action::LoginPassword => {
                        // Temporarily exit raw mode for interactive login
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;

                        println!("Login with username/password:");
                        if let Err(e) = login(cache, None, None).await {
                            eprintln!("Login failed: {}", e);
                            eprintln!("Press Enter to continue...");
                            let mut input = String::new();
                            let _ = io::stdin().read_line(&mut input);
                        } else {
                            println!("Press Enter to continue...");
                            let mut input = String::new();
                            let _ = io::stdin().read_line(&mut input);
                        }

                        // Re-enter TUI mode
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;

                        // Reload token and refresh data
                        if let Ok(token_data) = cache.load_token() {
                            client = ShkoloClient::with_token(token_data.token, token_data.school_year);
                            if let Some(data) = token_data.user_data {
                                if let Some(names) = data.get("names").and_then(|v: &serde_json::Value| v.as_str()) {
                                    app.user_name = Some(names.to_string());
                                } else if let Some(users) = data.get("users").and_then(|v: &serde_json::Value| v.as_array()) {
                                    if let Some(first) = users.first() {
                                        if let Some(names) = first.get("names").and_then(|v: &serde_json::Value| v.as_str()) {
                                            app.user_name = Some(names.to_string());
                                        }
                                    }
                                }
                            }
                            app.loading = true;
                            app.set_status("Loading data...");
                            terminal.draw(|f| draw(f, &app))?;
                            if let Err(e) = app.refresh_data(&client, cache, true).await {
                                app.set_status(format!("Error: {}", e));
                            }
                        }
                    }
                    Action::LoginGoogle => {
                        // Google login requires browser - show message
                        app.set_status("Google login not yet implemented in TUI");
                    }
                    Action::ImportToken => {
                        // Temporarily exit raw mode for import output
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;

                        if let Err(e) = import_token(cache) {
                            eprintln!("Import failed: {}", e);
                        }
                        println!("\nPress Enter to continue...");
                        let mut input = String::new();
                        let _ = io::stdin().read_line(&mut input);

                        // Re-enter TUI mode
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;

                        // Reload token and refresh data
                        if let Ok(token_data) = cache.load_token() {
                            client = ShkoloClient::with_token(token_data.token, token_data.school_year);
                            if let Some(data) = token_data.user_data {
                                if let Some(names) = data.get("names").and_then(|v: &serde_json::Value| v.as_str()) {
                                    app.user_name = Some(names.to_string());
                                }
                            }
                            app.loading = true;
                            app.set_status("Loading data...");
                            terminal.draw(|f| draw(f, &app))?;
                            if let Err(e) = app.refresh_data(&client, cache, true).await {
                                app.set_status(format!("Error: {}", e));
                            }
                        }
                    }
                    Action::None => {}
                },
                Event::Mouse(mouse) => {
                    // Handle mouse events for pane resizing
                    // The pane border is at x = students_pane_width (after the 3-line header)
                    let border_x = app.students_pane_width;
                    match mouse.kind {
                        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                            // Only resize if dragging near the border (within 2 chars)
                            if mouse.row >= 3 { // Skip header area
                                let new_width = mouse.column.clamp(15, 60);
                                app.students_pane_width = new_width;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if !app.running {
            break;
        }
    }

    // Save UI configuration (pane sizes)
    let ui_config = cache::UiConfig {
        students_pane_width: Some(app.students_pane_width),
    };
    let _ = cache.save_ui_config(&ui_config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;

    Ok(())
}

fn import_token(cache: &CacheStore) -> Result<()> {
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

    cache.save_token(token, None, Some(serde_json::json!({
        "names": user_name,
        "id": user_id,
    })))?;

    println!("Token imported successfully!");
    println!("User: {}", user_name);
    if let Some(id) = user_id {
        println!("User ID: {}", id);
    }

    Ok(())
}

async fn login(cache: &CacheStore, username: Option<String>, password: Option<String>) -> Result<()> {
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
        client.token().unwrap(),
        client.school_year(),
        Some(user_data.clone()),
    )?;

    println!("Logged in successfully!");

    if let Some(users) = response.users {
        for user in users {
            if let Some(name) = user.names {
                println!("  User: {}", name);
            }
            if let Some(roles) = user.roles {
                let role_names: Vec<_> = roles.iter()
                    .filter_map(|r| r.role_name.clone())
                    .collect();
                if !role_names.is_empty() {
                    println!("  Roles: {}", role_names.join(", "));
                }
            }
        }
    }

    Ok(())
}

async fn login_google(cache: &CacheStore, token: Option<String>) -> Result<()> {
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
        client.token().unwrap(),
        client.school_year(),
        Some(user_data),
    )?;

    println!("Logged in with Google successfully!");

    Ok(())
}

async fn logout(cache: &CacheStore) -> Result<()> {
    if let Ok(token_data) = cache.load_token() {
        let mut client = ShkoloClient::with_token(token_data.token, token_data.school_year);
        let _ = client.logout().await;
    }

    cache.clear_token()?;
    println!("Logged out successfully!");

    Ok(())
}

fn show_status(cache: &CacheStore) -> Result<()> {
    match cache.load_token() {
        Ok(token_data) => {
            println!("Status: Authenticated");

            if let Some(user_data) = token_data.user_data {
                if let Some(name) = user_data.get("names").and_then(|v| v.as_str()) {
                    println!("User: {}", name);
                }
                if let Some(users) = user_data.get("users").and_then(|v| v.as_array()) {
                    for user in users {
                        if let Some(name) = user.get("names").and_then(|v| v.as_str()) {
                            println!("User: {}", name);
                        }
                    }
                }
            }

            if let Some(year) = token_data.school_year {
                println!("School Year ID: {}", year);
            }

            println!();
            println!("Cache directory: {}", cache.cache_dir().display());
            println!("Cache TTL: {} seconds", cache.ttl());
        }
        Err(_) => {
            println!("Status: Not authenticated");
            println!();
            println!("Run 'shkolo login' or 'shkolo import-token' to authenticate");
        }
    }

    Ok(())
}

async fn cache_command(cache: &CacheStore, clear: bool, clear_all: bool, refresh: bool) -> Result<()> {
    if clear_all {
        cache.clear_all()?;
        println!("All cache cleared (including token)");
    } else if clear {
        cache.clear()?;
        println!("Cache cleared (token preserved)");
    }

    if refresh {
        let client = get_authenticated_client(cache)?;

        println!("Refreshing all data...");

        // Get students
        let pupils_response = client.get_pupils().await?;
        let mut students = Vec::new();

        if let Some(child_pupils) = pupils_response.child_pupils {
            for (id, pupil) in child_pupils {
                students.push(Student::from_child_pupil(&id, &pupil));
            }
        }

        cache.save_students(&students)?;
        println!("  Refreshed {} students", students.len());

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
                            let subject = course.course_short_name.or(course.course_name).unwrap_or_default();
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

            // Refresh grades
            if let Ok(grades_response) = client.get_grades_summary(student.id).await {
                let courses = grades_response.grades.or(grades_response.courses).unwrap_or_default();
                let grades: Vec<_> = courses.iter()
                    .map(Grade::from_course_grades)
                    .filter(|g| g.has_grades())
                    .collect();
                cache.save_grades(student.id, &grades)?;
            }

            // Refresh schedule
            if let Ok(schedule_response) = client.get_pupil_schedule(student.id, &today).await {
                let hours = schedule_response.schedule_hours.or(schedule_response.data).unwrap_or_default();
                let mut schedule: Vec<_> = hours.iter().map(ScheduleHour::from_raw).collect();
                schedule.sort_by_key(|h| h.hour_number);
                cache.save_schedule(student.id, &today, &schedule)?;
            }

            println!("  Refreshed data for {}", student.name);
        }

        println!("All data refreshed!");
    }

    if !clear && !clear_all && !refresh {
        println!("Cache directory: {}", cache.cache_dir().display());
        println!("Cache TTL: {} seconds", cache.ttl());
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

    Ok(ShkoloClient::with_token(token_data.token, token_data.school_year))
}

async fn get_students(
    client: &ShkoloClient,
    cache: &CacheStore,
    force_refresh: bool,
) -> Result<(Vec<Student>, bool, Option<String>)> {
    // Check cache first
    if !force_refresh {
        if let Some((students, age, expired)) = cache.get_students() {
            if !expired {
                return Ok((students, true, Some(age)));
            }
        }
    }

    // Fetch from API
    let pupils_response = client.get_pupils().await?;

    let mut students = Vec::new();
    if let Some(child_pupils) = pupils_response.child_pupils {
        for (id, pupil) in child_pupils {
            students.push(Student::from_child_pupil(&id, &pupil));
        }
    }

    students.sort_by(|a, b| a.name.cmp(&b.name));
    cache.save_students(&students)?;

    Ok((students, false, None))
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

    all_homework.sort_by(|a, b| b.date_sort.cmp(&a.date_sort));
    cache.save_homework(student_id, &all_homework)?;

    Ok((all_homework, false, None))
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

    // Fetch from API
    let response = client.get_grades_summary(student_id).await?;

    let courses = response.grades.or(response.courses).unwrap_or_default();
    let grades: Vec<Grade> = courses
        .iter()
        .map(Grade::from_course_grades)
        .filter(|g| g.has_grades())
        .collect();

    cache.save_grades(student_id, &grades)?;

    Ok((grades, false, None))
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

    Ok((schedule, false, None))
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
    let now = OffsetDateTime::now_utc();
    format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}
