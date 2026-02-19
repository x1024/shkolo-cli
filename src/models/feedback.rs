use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRaw {
    pub id: Option<i64>,
    pub badge_name: Option<String>,
    pub badge_icon: Option<String>,
    /// Badge type: 1 = positive, 2 = negative
    pub badge_type_id: Option<i64>,
    /// Comment/text
    pub text: Option<String>,
    /// Date in DD.MM.YYYY format
    pub created_date: Option<String>,
    /// Teacher name
    pub created_by: Option<String>,
    /// Subject name
    pub course_name: Option<String>,
    pub course_short_name: Option<String>,
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
        // Date is already in DD.MM.YYYY format from API
        let date = raw.created_date.clone().unwrap_or_else(|| "N/A".to_string());

        // badge_type_id: 1 = positive, 2 = negative
        let is_positive = raw.badge_type_id.map(|t| t == 1).unwrap_or(true);

        Self {
            id: raw.id.unwrap_or(0),
            badge_name: raw.badge_name.clone().unwrap_or_else(|| "Feedback".to_string()),
            badge_icon: raw.badge_icon.clone(),
            comment: raw.text.clone(),
            is_positive,
            date,
            teacher: raw.created_by.clone().unwrap_or_default(),
            subject: raw.course_short_name.clone()
                .or_else(|| raw.course_name.clone())
                .unwrap_or_default(),
        }
    }

    /// Get emoji for badge type - map badge_icon filenames to emojis
    pub fn emoji(&self) -> String {
        if let Some(ref icon) = self.badge_icon {
            if !icon.is_empty() {
                // Map known badge icon filenames to emojis
                return match icon.as_str() {
                    // Negative badges
                    "no-homework.png" | "no_homework.png" => "📝❌",
                    "not-prepared.png" | "unprepared.png" => "❌",
                    "no-attention.png" | "attention.png" => "👀",
                    "poor-performance.png" => "📉",
                    "disrespect.png" => "😠",
                    "bad-behavior.png" | "bad_behavior.png" => "👎",
                    "late.png" => "⏰",
                    "warning.png" => "⚠️",
                    // Positive badges
                    "excellence.png" | "excellent-work.png" => "🌟",
                    "creativity.png" => "🎨",
                    "homework.png" => "✅",
                    "active-participation.png" => "🙋",
                    "prepared.png" => "📚",
                    "good-behavior.png" | "behavior.png" => "👍",
                    "praise.png" => "🏆",
                    "thumbs-up.png" => "👍",
                    "star.png" => "⭐",
                    // If it looks like an emoji already (non-ASCII first char), use it
                    s if s.chars().next().map(|c| !c.is_ascii()).unwrap_or(false) => s,
                    // Unknown icon - fallback based on is_positive
                    _ => if self.is_positive { "⭐" } else { "⚠️" },
                }.to_string();
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
