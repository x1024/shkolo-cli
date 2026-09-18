use anyhow::{anyhow, Result};
use reqwest::{Client, header};
use serde::de::DeserializeOwned;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::models::*;
use super::types::*;

const API_BASE_URL: &str = "https://api.shkolo.bg";
const WEB_BASE_URL: &str = "https://app.shkolo.bg";
const USER_AGENT: &str = "Shkolo-app-iOS/1.43.3";
const WEB_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const GOOGLE_CLIENT_ID: &str = "186341692533-14k2gd4i6fsj230cqu40jf04dp0igr3j.apps.googleusercontent.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Bearer,
    Cookie,
}

/// Mutable auth state shared by all clones of a client. The server keeps the
/// selected user (school) and year per token, so every clone must see the same
/// view of "where the server currently is".
#[derive(Debug, Default)]
struct AuthState {
    token: Option<String>,
    school_year: Option<i64>,
    user_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ShkoloClient {
    client: Client,
    state: Arc<RwLock<AuthState>>,
    auth_mode: AuthMode,
    xsrf_token: Option<String>,
}

impl ShkoloClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            state: Arc::new(RwLock::new(AuthState::default())),
            auth_mode: AuthMode::Bearer,
            xsrf_token: None,
        }
    }

    pub fn with_token(token: String, school_year: Option<i64>) -> Self {
        let client = Self::new();
        client.set_token(token);
        if let Some(year) = school_year {
            client.set_school_year(year);
        }
        client
    }

    pub fn with_cookie(cookie: String, xsrf_token: Option<String>, school_year: Option<i64>) -> Self {
        let mut client = Self::new();
        client.set_token(cookie);
        client.auth_mode = AuthMode::Cookie;
        client.xsrf_token = xsrf_token;
        if let Some(year) = school_year {
            client.set_school_year(year);
        }
        client
    }

    pub fn token(&self) -> Option<String> {
        self.state.read().unwrap().token.clone()
    }

    pub fn school_year(&self) -> Option<i64> {
        self.state.read().unwrap().school_year
    }

    /// The school context (user id, school year) the server is believed to be on
    pub fn current_context(&self) -> Option<(i64, i64)> {
        let state = self.state.read().unwrap();
        Some((state.user_id?, state.school_year?))
    }

    pub fn set_school_year(&self, year_id: i64) {
        self.state.write().unwrap().school_year = Some(year_id);
    }

    pub fn set_token(&self, token: String) {
        self.state.write().unwrap().token = Some(token);
    }

    pub fn set_user_id(&self, user_id: Option<i64>) {
        self.state.write().unwrap().user_id = user_id;
    }

    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode
    }

    fn base_url(&self) -> &str {
        match self.auth_mode {
            AuthMode::Bearer => API_BASE_URL,
            AuthMode::Cookie => WEB_BASE_URL,
        }
    }

    fn headers(&self, authorized: bool) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert("language", "bg".parse().unwrap());

        let state = self.state.read().unwrap();
        match self.auth_mode {
            AuthMode::Bearer => {
                headers.insert(header::USER_AGENT, USER_AGENT.parse().unwrap());
                if authorized {
                    if let Some(ref token) = state.token {
                        headers.insert(
                            header::AUTHORIZATION,
                            format!("Bearer {}", token).parse().unwrap(),
                        );
                    }
                }
            }
            AuthMode::Cookie => {
                headers.insert(header::USER_AGENT, WEB_USER_AGENT.parse().unwrap());
                if authorized {
                    if let Some(ref cookie) = state.token {
                        headers.insert(
                            header::COOKIE,
                            format!("shkolo-cookie={}", cookie).parse().unwrap(),
                        );
                    }
                    if let Some(ref xsrf) = self.xsrf_token {
                        headers.insert("X-XSRF-TOKEN", xsrf.parse().unwrap());
                    }
                }
            }
        }

        if let Some(year) = state.school_year {
            headers.insert("School-Year", year.to_string().parse().unwrap());
        }

        headers
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url(), endpoint);
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
        let url = format!("{}{}", self.base_url(), endpoint);
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
        self.set_token(token);

        // Get users and years to select school year
        let users_response = self.get_users_and_years().await?;
        self.adopt_selected(&users_response);

        Ok(users_response)
    }

    /// Login with Google ID token
    pub async fn login_google(&mut self, id_token: &str) -> Result<UsersAndYearsResponse> {
        let request = GoogleAuthRequest {
            id_token: id_token.to_string(),
        };

        let response: LoginResponse = self.post("/v1/auth/google", &request, false).await?;

        let token = response.token.ok_or_else(|| anyhow!("No token received from Google auth"))?;
        self.set_token(token);

        // Get users and years
        let users_response = self.get_users_and_years().await?;
        self.adopt_selected(&users_response);

        Ok(users_response)
    }

    /// Take the server-selected user and year (or the latest year) as the current context
    fn adopt_selected(&self, users_response: &UsersAndYearsResponse) {
        if let Some(ref entries) = users_response.users {
            if let Some(selected) = entries.iter().find(|e| e.is_selected == Some(true)) {
                if let Some(year) = selected.school_year {
                    self.set_school_year(year);
                }
                self.set_user_id(selected.id);
            } else if let Some(max_year) = entries.iter().filter_map(|e| e.school_year).max() {
                self.set_school_year(max_year);
            }
        }
    }

    /// Logout from current session
    pub async fn logout(&mut self) -> Result<()> {
        if self.token().is_some() {
            let _: serde_json::Value = self.post("/v1/auth/logout", &serde_json::json!({}), true).await
                .unwrap_or(serde_json::json!({}));
        }
        *self.state.write().unwrap() = AuthState::default();
        Ok(())
    }

    /// Get users and years
    pub async fn get_users_and_years(&self) -> Result<UsersAndYearsResponse> {
        self.get("/v1/auth/usersAndYears").await
    }

    /// Switch the active parent user (i.e. school) and school year on the server.
    ///
    /// The selection is server-side state tied to the token: afterwards
    /// `/v1/diary/pupils` returns that school's pupils and usersAndYears marks the
    /// user+year as selected. Verified against the switch response and usersAndYears,
    /// or as a last resort by the pupils' school matching `expected_school`.
    /// On success the client's token (if a new one is issued) and School-Year
    /// header are updated. On failure the caller should restore the previous token/year.
    pub async fn switch_user_and_year(
        &self,
        user_id: i64,
        school_year: i64,
        expected_school: Option<&str>,
    ) -> Result<UsersAndYearsResponse> {
        // Body shape verified against the live API: `user_id` is required and the
        // year key is `year` (school_year / year_id / schoolYear are ignored).
        let body = serde_json::json!({
            "user_id": user_id,
            "year": school_year,
        });

        let response: serde_json::Value = self.post("/v1/auth/switchUserAndYear", &body, true).await?;
        // The response echoes the bearer token as `apiToken`; adopt it in case it ever rotates
        if let Some(token) = response.get("apiToken").or_else(|| response.get("token")).and_then(|v| v.as_str()) {
            if !token.is_empty() {
                self.set_token(token.to_string());
            }
        }
        self.set_school_year(school_year);

        let response_ok = response.get("user").and_then(|u| u.get("id")).and_then(|v| v.as_i64()) == Some(user_id)
            && response.get("selectedYear").and_then(|v| v.as_i64()) == Some(school_year);

        let users = self.get_users_and_years().await?;
        let selected_ok = response_ok || users.users.iter().flatten().any(|e| {
            e.id == Some(user_id) && e.school_year == Some(school_year) && e.is_selected == Some(true)
        });

        let pupils_ok = if selected_ok {
            true
        } else {
            match (expected_school, self.get_pupils().await) {
                (Some(school), Ok(pupils)) => pupils
                    .child_pupils
                    .iter()
                    .flat_map(|m| m.values())
                    .any(|p| p.school_name.as_deref() == Some(school)),
                _ => false,
            }
        };

        if !selected_ok && !pupils_ok {
            return Err(anyhow!(
                "Server did not confirm switch to user {} / school year {}",
                user_id, school_year
            ));
        }

        self.set_user_id(Some(user_id));
        Ok(users)
    }

    /// Switch to the given school context unless the server is already on it.
    /// Returns whether a switch was made.
    pub async fn ensure_context(&self, user_id: i64, school_year: i64) -> Result<bool> {
        if self.current_context() == Some((user_id, school_year)) {
            return Ok(false);
        }
        self.switch_user_and_year(user_id, school_year, None).await?;
        Ok(true)
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

    /// Get grades summary for a pupil (legacy, no dates/comments)
    pub async fn get_grades_summary(&self, pupil_id: i64) -> Result<GradesSummaryResponse> {
        self.get(&format!("/v1/diary/pupils/{}/grades/summary", pupil_id)).await
    }

    /// Get detailed grades for a pupil (includes dates, comments, types)
    pub async fn get_grades(&self, pupil_id: i64) -> Result<GradesListResponse> {
        self.get(&format!("/v1/diary/pupils/{}/grades", pupil_id)).await
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

    /// Verify that authentication works by probing the pupils endpoint
    pub async fn verify_auth(&self) -> Result<()> {
        let _: PupilsResponse = self.get("/v1/diary/pupils").await?;
        Ok(())
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

impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMode::Bearer => write!(f, "bearer"),
            AuthMode::Cookie => write!(f, "cookie"),
        }
    }
}
