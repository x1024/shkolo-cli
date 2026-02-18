use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};

use crate::models::*;

const DEFAULT_TTL_SECONDS: i64 = 3600; // 1 hour

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    pub timestamps: HashMap<String, i64>,
}

impl Default for CacheMeta {
    fn default() -> Self {
        Self {
            timestamps: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    pub students_pane_width: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub school_year: Option<i64>,
    pub user_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
    pub data: T,
    pub cached_at: i64, // Unix timestamp
}

impl<T> CachedData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            cached_at: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn is_expired(&self, ttl_seconds: i64) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let age = now - self.cached_at;
        age > ttl_seconds
    }

    pub fn age_string(&self) -> String {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let seconds = now - self.cached_at;

        if seconds < 60 {
            format!("{}s ago", seconds)
        } else if seconds < 3600 {
            format!("{}m ago", seconds / 60)
        } else if seconds < 86400 {
            format!("{}h ago", seconds / 3600)
        } else {
            format!("{}d ago", seconds / 86400)
        }
    }
}

#[derive(Debug)]
pub struct CacheStore {
    cache_dir: PathBuf,
    ttl_seconds: i64,
}

impl CacheStore {
    pub fn new(ttl_seconds: Option<i64>) -> Result<Self> {
        let home = dirs_home();
        let cache_dir = home.join(".shkolo").join("cache");
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            ttl_seconds: ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS),
        })
    }

    pub fn config_dir() -> PathBuf {
        dirs_home().join(".shkolo")
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    pub fn ttl(&self) -> i64 {
        self.ttl_seconds
    }

    fn file_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", name))
    }

    fn read_file<T: DeserializeOwned>(&self, name: &str) -> Result<T> {
        let path = self.file_path(name);
        let content = fs::read_to_string(&path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn write_file<T: Serialize>(&self, name: &str, data: &T) -> Result<()> {
        let path = self.file_path(name);
        let content = serde_json::to_string_pretty(data)?;
        fs::write(&path, content)?;

        // Set restrictive permissions on sensitive files
        #[cfg(unix)]
        if name == "token" {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    // Token management

    pub fn load_token(&self) -> Result<TokenData> {
        self.read_file("token")
    }

    pub fn save_token(&self, token: &str, school_year: Option<i64>, user_data: Option<serde_json::Value>) -> Result<()> {
        let data = TokenData {
            token: token.to_string(),
            school_year,
            user_data,
        };
        self.write_file("token", &data)
    }

    pub fn clear_token(&self) -> Result<()> {
        let path = self.file_path("token");
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    // Students cache

    pub fn load_students(&self) -> Result<CachedData<Vec<Student>>> {
        self.read_file("students")
    }

    pub fn save_students(&self, students: &[Student]) -> Result<()> {
        let cached = CachedData::new(students.to_vec());
        self.write_file("students", &cached)
    }

    pub fn get_students(&self) -> Option<(Vec<Student>, String, bool)> {
        match self.load_students() {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Homework cache (per student)

    pub fn load_homework(&self, student_id: i64) -> Result<CachedData<Vec<Homework>>> {
        self.read_file(&format!("homework_{}", student_id))
    }

    pub fn save_homework(&self, student_id: i64, homework: &[Homework]) -> Result<()> {
        let cached = CachedData::new(homework.to_vec());
        self.write_file(&format!("homework_{}", student_id), &cached)
    }

    pub fn get_homework(&self, student_id: i64) -> Option<(Vec<Homework>, String, bool)> {
        match self.load_homework(student_id) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Grades cache (per student)

    pub fn load_grades(&self, student_id: i64) -> Result<CachedData<Vec<Grade>>> {
        self.read_file(&format!("grades_{}", student_id))
    }

    pub fn save_grades(&self, student_id: i64, grades: &[Grade]) -> Result<()> {
        let cached = CachedData::new(grades.to_vec());
        self.write_file(&format!("grades_{}", student_id), &cached)
    }

    pub fn get_grades(&self, student_id: i64) -> Option<(Vec<Grade>, String, bool)> {
        match self.load_grades(student_id) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Schedule cache (per student, per date)

    pub fn load_schedule(&self, student_id: i64, date: &str) -> Result<CachedData<Vec<ScheduleHour>>> {
        self.read_file(&format!("schedule_{}_{}", student_id, date))
    }

    pub fn save_schedule(&self, student_id: i64, date: &str, schedule: &[ScheduleHour]) -> Result<()> {
        let cached = CachedData::new(schedule.to_vec());
        self.write_file(&format!("schedule_{}_{}", student_id, date), &cached)
    }

    pub fn get_schedule(&self, student_id: i64, date: &str) -> Option<(Vec<ScheduleHour>, String, bool)> {
        match self.load_schedule(student_id, date) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Events cache (per student)

    pub fn load_events(&self, student_id: i64) -> Result<CachedData<Vec<Event>>> {
        self.read_file(&format!("events_{}", student_id))
    }

    pub fn save_events(&self, student_id: i64, events: &[Event]) -> Result<()> {
        let cached = CachedData::new(events.to_vec());
        self.write_file(&format!("events_{}", student_id), &cached)
    }

    pub fn get_events(&self, student_id: i64) -> Option<(Vec<Event>, String, bool)> {
        match self.load_events(student_id) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Notifications cache (global, not per student)

    pub fn load_notifications(&self) -> Result<CachedData<Vec<Notification>>> {
        self.read_file("notifications")
    }

    pub fn save_notifications(&self, notifications: &[Notification]) -> Result<()> {
        let cached = CachedData::new(notifications.to_vec());
        self.write_file("notifications", &cached)
    }

    pub fn get_notifications(&self) -> Option<(Vec<Notification>, String, bool)> {
        match self.load_notifications() {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Absences cache (per student)

    pub fn load_absences(&self, student_id: i64) -> Result<CachedData<Vec<Absence>>> {
        self.read_file(&format!("absences_{}", student_id))
    }

    pub fn save_absences(&self, student_id: i64, absences: &[Absence]) -> Result<()> {
        let cached = CachedData::new(absences.to_vec());
        self.write_file(&format!("absences_{}", student_id), &cached)
    }

    pub fn get_absences(&self, student_id: i64) -> Option<(Vec<Absence>, String, bool)> {
        match self.load_absences(student_id) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Messages cache (global, not per student)

    pub fn load_messages(&self) -> Result<CachedData<Vec<MessageThread>>> {
        self.read_file("messages")
    }

    pub fn save_messages(&self, messages: &[MessageThread]) -> Result<()> {
        let cached = CachedData::new(messages.to_vec());
        self.write_file("messages", &cached)
    }

    pub fn get_messages(&self) -> Option<(Vec<MessageThread>, String, bool)> {
        match self.load_messages() {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Feedbacks cache (per student)

    pub fn load_feedbacks(&self, student_id: i64) -> Result<CachedData<Vec<Feedback>>> {
        self.read_file(&format!("feedbacks_{}", student_id))
    }

    pub fn save_feedbacks(&self, student_id: i64, feedbacks: &[Feedback]) -> Result<()> {
        let cached = CachedData::new(feedbacks.to_vec());
        self.write_file(&format!("feedbacks_{}", student_id), &cached)
    }

    pub fn get_feedbacks(&self, student_id: i64) -> Option<(Vec<Feedback>, String, bool)> {
        match self.load_feedbacks(student_id) {
            Ok(cached) => {
                let expired = cached.is_expired(self.ttl_seconds);
                let age = cached.age_string();
                Some((cached.data, age, expired))
            }
            Err(_) => None,
        }
    }

    // Cache management

    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    // Don't delete token file on regular clear
                    if path.file_stem().map_or(false, |s| s != "token") {
                        fs::remove_file(path)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    // UI configuration (persistent settings)

    pub fn load_ui_config(&self) -> UiConfig {
        self.read_file::<UiConfig>("ui_config").unwrap_or_default()
    }

    pub fn save_ui_config(&self, config: &UiConfig) -> Result<()> {
        self.write_file("ui_config", config)
    }
}

fn dirs_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

// Add dirs crate functions since we're using directories
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}
