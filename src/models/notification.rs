use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<i64>,
    pub title: String,
    pub body: Option<String>,
    pub date: String,
    pub is_read: bool,
    pub notification_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRaw {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub message: Option<String>,
    pub created_at: Option<String>,
    pub date: Option<String>,
    pub is_read: Option<bool>,
    pub read: Option<bool>,
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
        Self {
            id: raw.id,
            title: raw.title.clone()
                .or_else(|| raw.subject.clone())
                .unwrap_or_else(|| "No title".to_string()),
            body: raw.body.clone().or_else(|| raw.message.clone()),
            date: raw.created_at.clone()
                .or_else(|| raw.date.clone())
                .unwrap_or_default(),
            is_read: raw.is_read.or(raw.read).unwrap_or(false),
            notification_type: raw.notification_type.clone(),
        }
    }
}
