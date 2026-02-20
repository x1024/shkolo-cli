use anyhow::{anyhow, Result};
use reqwest::{Client, header};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::models::*;
use super::types::*;

const API_BASE_URL: &str = "https://api.shkolo.bg";
const USER_AGENT: &str = "Shkolo-app-iOS/1.43.3";
const GOOGLE_CLIENT_ID: &str = "186341692533-14k2gd4i6fsj230cqu40jf04dp0igr3j.apps.googleusercontent.com";

#[derive(Debug, Clone)]
pub struct ShkoloClient {
    client: Client,
    token: Option<String>,
    school_year: Option<i64>,
}

impl ShkoloClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: None,
            school_year: None,
        }
    }

    pub fn with_token(token: String, school_year: Option<i64>) -> Self {
        let mut client = Self::new();
        client.token = Some(token);
        client.school_year = school_year;
        client
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn school_year(&self) -> Option<i64> {
        self.school_year
    }

    fn headers(&self, authorized: bool) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(header::USER_AGENT, USER_AGENT.parse().unwrap());
        headers.insert("language", "bg".parse().unwrap());

        if authorized {
            if let Some(ref token) = self.token {
                headers.insert(
                    header::AUTHORIZATION,
                    format!("Bearer {}", token).parse().unwrap(),
                );
            }
        }

        if let Some(year) = self.school_year {
            headers.insert("School-Year", year.to_string().parse().unwrap());
        }

        headers
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        let response = self.client
            .get(&url)
            .headers(self.headers(true))
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!("Session expired. Please login again."));
        }

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error ({}): {}", status, text));
        }

        let data = response.json().await?;
        Ok(data)
    }

    async fn post<T: DeserializeOwned, R: serde::Serialize>(&self, endpoint: &str, body: &R, authorized: bool) -> Result<T> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        let response = self.client
            .post(&url)
            .headers(self.headers(authorized))
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!("Session expired. Please login again."));
        }

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error ({}): {}", status, text));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Login with username and password
    pub async fn login(&mut self, username: &str, password: &str) -> Result<UsersAndYearsResponse> {
        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response: LoginResponse = self.post("/v1/auth/login", &request, false).await?;

        let token = response.token.ok_or_else(|| anyhow!("No token received"))?;
        self.token = Some(token);

        // Get users and years to select school year
        let users_response = self.get_users_and_years().await?;

        // Auto-select first available school year
        if let Some(ref users) = users_response.users {
            for user in users {
                if let Some(ref years) = user.years {
                    if let Some(year) = years.iter().max_by_key(|y| y.id) {
                        self.school_year = Some(year.id);
                        break;
                    }
                }
            }
        }

        Ok(users_response)
    }

    /// Login with Google ID token
    pub async fn login_google(&mut self, id_token: &str) -> Result<UsersAndYearsResponse> {
        let request = GoogleAuthRequest {
            id_token: id_token.to_string(),
        };

        let response: LoginResponse = self.post("/v1/auth/google", &request, false).await?;

        let token = response.token.ok_or_else(|| anyhow!("No token received from Google auth"))?;
        self.token = Some(token);

        // Get users and years
        let users_response = self.get_users_and_years().await?;

        // Auto-select first available school year
        if let Some(ref users) = users_response.users {
            for user in users {
                if let Some(ref years) = user.years {
                    if let Some(year) = years.iter().max_by_key(|y| y.id) {
                        self.school_year = Some(year.id);
                        break;
                    }
                }
            }
        }

        Ok(users_response)
    }

    /// Logout from current session
    pub async fn logout(&mut self) -> Result<()> {
        if self.token.is_some() {
            let _: serde_json::Value = self.post("/v1/auth/logout", &serde_json::json!({}), true).await
                .unwrap_or(serde_json::json!({}));
        }
        self.token = None;
        self.school_year = None;
        Ok(())
    }

    /// Get users and years
    pub async fn get_users_and_years(&self) -> Result<UsersAndYearsResponse> {
        self.get("/v1/auth/usersAndYears").await
    }

    /// Get pupils (children for parent accounts)
    pub async fn get_pupils(&self) -> Result<PupilsResponse> {
        self.get("/v1/diary/pupils").await
    }

    /// Get homework courses for a pupil
    pub async fn get_homework_courses(&self, pupil_id: i64) -> Result<HomeworkCoursesResponse> {
        self.get(&format!("/v1/diary/homeworks/courses?pupilId={}", pupil_id)).await
    }

    /// Get homework list for a course/class year
    pub async fn get_homework_list(&self, cyc_group_id: i64) -> Result<HomeworkListResponse> {
        self.get(&format!("/v1/diary/homeworks/list/{}", cyc_group_id)).await
    }

    /// Get grades summary for a pupil
    pub async fn get_grades_summary(&self, pupil_id: i64) -> Result<GradesSummaryResponse> {
        self.get(&format!("/v1/diary/pupils/{}/grades/summary", pupil_id)).await
    }

    /// Get schedule for a pupil on a specific date
    pub async fn get_pupil_schedule(&self, pupil_id: i64, date: &str) -> Result<ScheduleResponse> {
        self.get(&format!("/v1/diary/pupils/{}/scheduleHours?date={}", pupil_id, date)).await
    }

    /// Get events/invitations for a pupil (includes upcoming tests)
    pub async fn get_pupil_events(&self, pupil_id: i64) -> Result<EventsResponse> {
        self.get(&format!("/v1/events/invitations?pupil_user_id={}", pupil_id)).await
    }

    /// Get notifications
    pub async fn get_notifications(&self, page: i32) -> Result<NotificationsResponse> {
        self.get(&format!("/v1/notifications?page={}", page)).await
    }

    /// Get Google OAuth client ID
    pub fn google_client_id() -> &'static str {
        GOOGLE_CLIENT_ID
    }

    /// Get absences for a pupil
    pub async fn get_absences(&self, pupil_id: i64) -> Result<AbsencesResponse> {
        self.get(&format!("/v1/diary/pupils/{}/absences", pupil_id)).await
    }

    /// Get feedbacks (badges/remarks) for a pupil
    pub async fn get_feedbacks(&self, pupil_id: i64) -> Result<FeedbacksResponse> {
        self.get(&format!("/v1/diary/pupils/{}/feedbacks", pupil_id)).await
    }

    /// Get raw feedbacks response for debugging
    pub async fn get_feedbacks_raw(&self, pupil_id: i64) -> Result<serde_json::Value> {
        self.get(&format!("/v1/diary/pupils/{}/feedbacks", pupil_id)).await
    }

    // Messenger (Chat/Messages)

    /// Get message folders
    pub async fn get_messenger_folders(&self) -> Result<Vec<MessageFolder>> {
        let response: serde_json::Value = self.get("/v1/messenger/folders").await?;
        // The response is directly an array of folders
        let folders: Vec<MessageFolder> = serde_json::from_value(response)?;
        Ok(folders)
    }

    /// Get threads in a folder
    pub async fn get_messenger_threads(&self, folder_id: Option<i64>) -> Result<Vec<MessageThreadRaw>> {
        let response: serde_json::Value = match folder_id {
            Some(id) => self.get(&format!("/v1/messenger/threads?folderId={}", id)).await?,
            None => self.get("/v1/messenger/threads").await?,
        };
        // The response is directly an array of threads
        let threads: Vec<MessageThreadRaw> = serde_json::from_value(response)?;
        Ok(threads)
    }

    /// Get messages from a thread
    pub async fn get_thread_messages(&self, thread_id: i64) -> Result<Vec<Message>> {
        let response: serde_json::Value = self.get(&format!("/v1/messenger/threads/{}/messages", thread_id)).await?;

        // Response has "messages" array
        let messages_raw: Vec<MessageRaw> = if let Some(messages) = response.get("messages") {
            serde_json::from_value(messages.clone()).unwrap_or_default()
        } else if response.is_array() {
            // Response might be directly an array of messages
            serde_json::from_value(response.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(messages_raw.iter().map(Message::from_raw).collect())
    }

    /// Get raw thread data for debugging
    pub async fn get_thread_raw(&self, thread_id: i64) -> Result<serde_json::Value> {
        self.get(&format!("/v1/messenger/threads/{}/messages", thread_id)).await
    }

    /// Reply to an existing thread
    pub async fn reply_to_thread(&self, thread_id: i64, body: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "body": body
        });
        self.post(&format!("/v1/messenger/threads/{}", thread_id), &payload, true).await
    }

    /// Get available recipients for composing messages
    pub async fn get_recipients(&self) -> Result<Vec<Recipient>> {
        let response: serde_json::Value = self.get("/v1/messenger/recipients").await?;

        // API returns "users" array (not "recipients")
        let recipients_raw: Vec<RecipientRaw> = if response.is_array() {
            serde_json::from_value(response)?
        } else {
            // Try "users" first (actual API response), then "recipients" as fallback
            response.get("users")
                .or_else(|| response.get("recipients"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default()
        };

        Ok(recipients_raw.iter().map(Recipient::from_raw).collect())
    }

    /// Create a new message thread
    pub async fn create_thread(&self, recipient_ids: &[i64], subject: &str, body: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "recipient_ids": recipient_ids,
            "subject": subject,
            "body": body
        });
        self.post("/v1/messenger/threads", &payload, true).await
    }

    /// Check if user can send messages
    pub async fn can_send_messages(&self) -> Result<bool> {
        let response: serde_json::Value = self.get("/v1/messenger/canSendMessages").await?;
        Ok(response.get("canSendMessages").and_then(|v| v.as_bool()).unwrap_or(false))
    }
}

impl Default for ShkoloClient {
    fn default() -> Self {
        Self::new()
    }
}
