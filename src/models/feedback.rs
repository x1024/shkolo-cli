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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_positive_badge_type() {
        let raw = FeedbackRaw {
            id: Some(1),
            badge_name: Some("Excellent work".to_string()),
            badge_icon: Some("excellence.png".to_string()),
            badge_type_id: Some(1), // 1 = positive
            text: Some("Great job!".to_string()),
            created_date: Some("19.02.2026".to_string()),
            created_by: Some("Teacher Name".to_string()),
            course_name: Some("Mathematics".to_string()),
            course_short_name: Some("Math".to_string()),
        };

        let feedback = Feedback::from_raw(&raw);

        assert!(feedback.is_positive);
        assert_eq!(feedback.badge_name, "Excellent work");
        assert_eq!(feedback.date, "19.02.2026");
        assert_eq!(feedback.subject, "Math"); // Uses short name if available
    }

    #[test]
    fn test_feedback_negative_badge_type() {
        let raw = FeedbackRaw {
            id: Some(2),
            badge_name: Some("No homework".to_string()),
            badge_icon: Some("no-homework.png".to_string()),
            badge_type_id: Some(2), // 2 = negative
            text: Some("Missing homework".to_string()),
            created_date: Some("18.02.2026".to_string()),
            created_by: Some("Teacher Name".to_string()),
            course_name: Some("English".to_string()),
            course_short_name: None,
        };

        let feedback = Feedback::from_raw(&raw);

        assert!(!feedback.is_positive);
        assert_eq!(feedback.subject, "English"); // Falls back to course_name
    }

    #[test]
    fn test_feedback_default_positive_when_missing_type() {
        let raw = FeedbackRaw {
            id: Some(3),
            badge_name: Some("Unknown".to_string()),
            badge_icon: None,
            badge_type_id: None, // Missing type defaults to positive
            text: None,
            created_date: None,
            created_by: None,
            course_name: None,
            course_short_name: None,
        };

        let feedback = Feedback::from_raw(&raw);

        assert!(feedback.is_positive); // Defaults to positive
        assert_eq!(feedback.date, "N/A"); // Missing date shows N/A
    }

    #[test]
    fn test_feedback_emoji_mapping() {
        // Test positive badge icon
        let positive = Feedback {
            id: 1,
            badge_name: "Excellent".to_string(),
            badge_icon: Some("excellence.png".to_string()),
            comment: None,
            is_positive: true,
            date: "19.02.2026".to_string(),
            teacher: "Teacher".to_string(),
            subject: "Math".to_string(),
        };
        assert_eq!(positive.emoji(), "🌟");

        // Test negative badge icon
        let negative = Feedback {
            id: 2,
            badge_name: "No homework".to_string(),
            badge_icon: Some("no-homework.png".to_string()),
            comment: None,
            is_positive: false,
            date: "19.02.2026".to_string(),
            teacher: "Teacher".to_string(),
            subject: "Math".to_string(),
        };
        assert_eq!(negative.emoji(), "📝❌");

        // Test fallback for unknown icon
        let unknown = Feedback {
            id: 3,
            badge_name: "Unknown".to_string(),
            badge_icon: Some("unknown-icon.png".to_string()),
            comment: None,
            is_positive: true,
            date: "19.02.2026".to_string(),
            teacher: "Teacher".to_string(),
            subject: "Math".to_string(),
        };
        assert_eq!(unknown.emoji(), "⭐"); // Falls back to positive default

        // Test fallback for no icon
        let no_icon = Feedback {
            id: 4,
            badge_name: "Plain".to_string(),
            badge_icon: None,
            comment: None,
            is_positive: false,
            date: "19.02.2026".to_string(),
            teacher: "Teacher".to_string(),
            subject: "Math".to_string(),
        };
        assert_eq!(no_icon.emoji(), "⚠️"); // Falls back to negative default
    }
}
