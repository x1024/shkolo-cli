use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub date: String,
    pub is_read: bool,
    pub notification_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRaw {
    pub id: Option<String>,
    // Main text field from API
    pub text: Option<String>,
    // Alternative field names for compatibility
    pub title: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub message: Option<String>,
    // Timestamps
    pub created_at: Option<String>,
    pub date: Option<String>,
    // Read status - seen_at being non-null means it's read
    pub seen_at: Option<String>,
    pub is_read: Option<bool>,
    pub read: Option<bool>,
    // Type of notification
    pub notification_trigger_slug: Option<String>,
    #[serde(rename = "type")]
    pub notification_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsResponse {
    pub data: Option<Vec<NotificationRaw>>,
    pub notifications: Option<Vec<NotificationRaw>>,
}

impl Notification {
    pub fn from_raw(raw: &NotificationRaw) -> Self {
        // Determine read status - seen_at being present means it's read
        let is_read = raw.seen_at.is_some()
            || raw.is_read.unwrap_or(false)
            || raw.read.unwrap_or(false);

        Self {
            id: raw.id.clone(),
            title: raw.text.clone()
                .or_else(|| raw.title.clone())
                .or_else(|| raw.subject.clone())
                .unwrap_or_else(|| "No title".to_string()),
            body: raw.body.clone().or_else(|| raw.message.clone()),
            date: raw.created_at.clone()
                .or_else(|| raw.date.clone())
                .unwrap_or_default(),
            is_read,
            notification_type: raw.notification_trigger_slug.clone()
                .or_else(|| raw.notification_type.clone()),
        }
    }
}
