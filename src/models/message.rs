use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFolder {
    pub id: i64,
    pub name: String,
    pub system_folder_slug: Option<String>,
    pub folder_total_count: i32,
    pub folder_unread_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFoldersResponse {
    #[serde(default)]
    pub folders: Option<Vec<MessageFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThreadRaw {
    pub id: Option<i64>,
    pub subject: Option<String>,
    pub last_msg_body: Option<String>,
    pub last_msg_user: Option<String>,
    pub last_msg_user_id: Option<i64>,
    pub participant_count: Option<i32>,
    pub is_unread: Option<bool>,
    pub is_draft: Option<i32>,
    pub updated_at: Option<String>,
    pub thread_creator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThreadsResponse {
    #[serde(default)]
    pub threads: Option<Vec<MessageThreadRaw>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThread {
    pub id: i64,
    pub subject: String,
    pub last_message: String,
    pub last_sender: String,
    pub participant_count: i32,
    pub is_unread: bool,
    pub updated_at: String,
    pub creator: String,
}

impl MessageThread {
    pub fn from_raw(raw: &MessageThreadRaw) -> Self {
        Self {
            id: raw.id.unwrap_or(0),
            subject: raw.subject.clone().unwrap_or_default(),
            last_message: raw.last_msg_body.clone().unwrap_or_default(),
            last_sender: raw.last_msg_user.clone().unwrap_or_default(),
            participant_count: raw.participant_count.unwrap_or(0),
            is_unread: raw.is_unread.unwrap_or(false),
            updated_at: raw.updated_at.clone().unwrap_or_default(),
            creator: raw.thread_creator.clone().unwrap_or_default(),
        }
    }

    /// Format the update time for display (extract date or time)
    pub fn display_time(&self) -> String {
        // Format: "2026-02-18 09:47:18" -> extract relevant part
        if self.updated_at.len() >= 16 {
            // Return "DD.MM HH:MM" format
            let parts: Vec<&str> = self.updated_at.split(' ').collect();
            if parts.len() >= 2 {
                let date_parts: Vec<&str> = parts[0].split('-').collect();
                if date_parts.len() == 3 {
                    let time = if parts[1].len() >= 5 { &parts[1][..5] } else { parts[1] };
                    return format!("{}.{} {}", date_parts[2], date_parts[1], time);
                }
            }
        }
        self.updated_at.clone()
    }

    /// Truncate the last message for preview
    pub fn preview(&self, max_len: usize) -> String {
        if self.last_message.len() <= max_len {
            self.last_message.clone()
        } else {
            format!("{}...", &self.last_message[..max_len.saturating_sub(3)])
        }
    }
}
