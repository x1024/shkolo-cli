use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFolder {
    pub id: i64,
    pub name: String,
    pub system_folder_slug: Option<String>,
    pub folder_total_count: i32,
    pub folder_unread_count: i32,
}

/// Raw message within a thread (from API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRaw {
    pub id: Option<i64>,
    pub body: Option<String>,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub user_names: Option<String>,
    pub created_at: Option<String>,
    pub is_system: Option<bool>,
}

/// A single message within a conversation thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub body: String,
    pub sender_id: i64,
    pub sender_name: String,
    pub date: String,
    pub is_system: bool,
}

impl Message {
    pub fn from_raw(raw: &MessageRaw) -> Self {
        Self {
            id: raw.id.unwrap_or(0),
            body: raw.body.clone().unwrap_or_default(),
            sender_id: raw.user_id.unwrap_or(0),
            sender_name: raw.user_names.clone()
                .or_else(|| raw.user_name.clone())
                .unwrap_or_default(),
            date: Self::format_date(raw.created_at.as_deref()),
            is_system: raw.is_system.unwrap_or(false),
        }
    }

    fn format_date(date_str: Option<&str>) -> String {
        match date_str {
            Some(d) if d.len() >= 16 => {
                // Format: "2026-02-18 09:47:18" -> "18.02.2026 09:47"
                let parts: Vec<&str> = d.split(' ').collect();
                if parts.len() >= 2 {
                    let date_parts: Vec<&str> = parts[0].split('-').collect();
                    if date_parts.len() == 3 {
                        let time: String = parts[1].chars().take(5).collect();
                        return format!("{}.{}.{} {}", date_parts[2], date_parts[1], date_parts[0], time);
                    }
                }
                d.to_string()
            }
            Some(d) => d.to_string(),
            None => String::new(),
        }
    }
}

/// Recipient for composing new messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientRaw {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub names: Option<String>,
    #[serde(rename = "type")]
    pub recipient_type: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub id: i64,
    pub name: String,
    pub recipient_type: String,
}

impl Recipient {
    pub fn from_raw(raw: &RecipientRaw) -> Self {
        Self {
            id: raw.id.unwrap_or(0),
            name: raw.names.clone()
                .or_else(|| raw.name.clone())
                .unwrap_or_default(),
            recipient_type: raw.recipient_type.clone().unwrap_or_default(),
        }
    }
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
        let parts: Vec<&str> = self.updated_at.split(' ').collect();
        if parts.len() >= 2 {
            let date_parts: Vec<&str> = parts[0].split('-').collect();
            if date_parts.len() == 3 {
                // Safely extract time (first 5 chars)
                let time: String = parts[1].chars().take(5).collect();
                return format!("{}.{} {}", date_parts[2], date_parts[1], time);
            }
        }
        self.updated_at.clone()
    }

    /// Truncate the last message for preview (UTF-8 safe)
    pub fn preview(&self, max_len: usize) -> String {
        let char_count = self.last_message.chars().count();
        if char_count <= max_len {
            self.last_message.clone()
        } else {
            let truncated: String = self.last_message.chars().take(max_len.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }
}
