use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRaw {
    pub id: Option<i64>,
    pub badge_id: Option<i64>,
    pub badge_name: Option<String>,
    pub badge_icon: Option<String>,
    pub comment: Option<String>,
    pub is_positive: Option<bool>,
    pub created_at: Option<String>,
    pub teacher_name: Option<String>,
    pub teacher_names: Option<String>,
    pub subject_name: Option<String>,
    pub course_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbacksResponse {
    pub data: Option<Vec<FeedbackRaw>>,
    pub feedbacks: Option<Vec<FeedbackRaw>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: i64,
    pub badge_name: String,
    pub badge_icon: Option<String>,
    pub comment: Option<String>,
    pub is_positive: bool,
    pub date: String,
    pub teacher: String,
    pub subject: String,
}

impl Feedback {
    pub fn from_raw(raw: &FeedbackRaw) -> Self {
        Self {
            id: raw.id.unwrap_or(0),
            badge_name: raw.badge_name.clone().unwrap_or_else(|| "Feedback".to_string()),
            badge_icon: raw.badge_icon.clone(),
            comment: raw.comment.clone(),
            is_positive: raw.is_positive.unwrap_or(true),
            date: raw.created_at.clone()
                .map(|d| {
                    // Format: "2026-02-18 09:47:18" -> "18.02.2026"
                    if d.len() >= 10 {
                        let parts: Vec<&str> = d[..10].split('-').collect();
                        if parts.len() == 3 {
                            return format!("{}.{}.{}", parts[2], parts[1], parts[0]);
                        }
                    }
                    d
                })
                .unwrap_or_default(),
            teacher: raw.teacher_names.clone()
                .or_else(|| raw.teacher_name.clone())
                .unwrap_or_default(),
            subject: raw.course_name.clone()
                .or_else(|| raw.subject_name.clone())
                .unwrap_or_default(),
        }
    }

    /// Get emoji for badge type - use badge_icon if available, otherwise based on is_positive
    pub fn emoji(&self) -> String {
        if let Some(ref icon) = self.badge_icon {
            if !icon.is_empty() {
                return icon.clone();
            }
        }
        // Fallback based on is_positive
        if self.is_positive {
            "⭐".to_string()
        } else {
            "⚠️".to_string()
        }
    }
}
