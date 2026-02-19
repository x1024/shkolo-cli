use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Homework {
    pub id: Option<i64>,
    pub subject: String,
    pub text: String,
    pub date: String,
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeworkCourse {
    pub cyc_group_id: Option<i64>,
    pub course_name: Option<String>,
    pub course_short_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeworkCoursesResponse {
    pub courses: Option<Vec<HomeworkCourse>>,
    #[serde(rename = "cycGroupHomeworksCount")]
    pub cyc_group_homeworks_count: Option<std::collections::HashMap<String, i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeworkItem {
    pub id: Option<i64>,
    pub homework_text: Option<String>,
    pub homework_due_date: Option<String>,
    pub shi_date: Option<String>,
    #[serde(rename = "shi_date_for_sort")]
    pub shi_date_for_sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeworkListResponse {
    pub homeworks: Option<Vec<HomeworkItem>>,
}

impl Homework {
    pub fn from_item(item: &HomeworkItem, subject: &str) -> Self {
        // Convert due date from DD.MM.YYYY to YYYY-MM-DD for sorting
        let due_date_sort = item.homework_due_date.as_ref().and_then(|d| {
            let parts: Vec<&str> = d.split('.').collect();
            if parts.len() == 3 {
                Some(format!("{}-{}-{}", parts[2], parts[1], parts[0]))
            } else {
                None
            }
        });

        Self {
            id: item.id,
            subject: subject.to_string(),
            text: item.homework_text.clone().unwrap_or_default(),
            date: item.shi_date.clone().unwrap_or_default(),
            due_date: item.homework_due_date.clone(),
            date_sort: item.shi_date_for_sort.clone(),
            due_date_sort,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_due_date_parsing() {
        let item = HomeworkItem {
            id: Some(1),
            homework_text: Some("Do math".to_string()),
            homework_due_date: Some("25.02.2026".to_string()),
            shi_date: Some("20.02.2026".to_string()),
            shi_date_for_sort: Some("2026-02-20".to_string()),
        };

        let hw = Homework::from_item(&item, "Math");

        assert_eq!(hw.due_date, Some("25.02.2026".to_string()));
        assert_eq!(hw.due_date_sort, Some("2026-02-25".to_string()));
    }

    #[test]
    fn test_due_date_parsing_invalid() {
        let item = HomeworkItem {
            id: Some(1),
            homework_text: Some("Do math".to_string()),
            homework_due_date: Some("invalid-date".to_string()),
            shi_date: None,
            shi_date_for_sort: None,
        };

        let hw = Homework::from_item(&item, "Math");

        assert_eq!(hw.due_date, Some("invalid-date".to_string()));
        assert_eq!(hw.due_date_sort, None); // Invalid format returns None
    }

    #[test]
    fn test_homework_sorting_future_ascending() {
        // Future homework should be sorted by due_date ascending (soonest first)
        let mut homework = vec![
            Homework {
                id: Some(1),
                subject: "Math".to_string(),
                text: "HW 1".to_string(),
                date: "20.02.2026".to_string(),
                due_date: Some("28.02.2026".to_string()),
                date_sort: Some("2026-02-20".to_string()),
                due_date_sort: Some("2026-02-28".to_string()),
            },
            Homework {
                id: Some(2),
                subject: "English".to_string(),
                text: "HW 2".to_string(),
                date: "20.02.2026".to_string(),
                due_date: Some("22.02.2026".to_string()),
                date_sort: Some("2026-02-20".to_string()),
                due_date_sort: Some("2026-02-22".to_string()),
            },
            Homework {
                id: Some(3),
                subject: "History".to_string(),
                text: "HW 3".to_string(),
                date: "20.02.2026".to_string(),
                due_date: Some("25.02.2026".to_string()),
                date_sort: Some("2026-02-20".to_string()),
                due_date_sort: Some("2026-02-25".to_string()),
            },
        ];

        // Sort ascending by due_date (soonest first)
        homework.sort_by(|a, b| {
            let a_due = a.due_date_sort.as_deref().unwrap_or("9999-99-99");
            let b_due = b.due_date_sort.as_deref().unwrap_or("9999-99-99");
            a_due.cmp(b_due)
        });

        assert_eq!(homework[0].subject, "English"); // 22nd - soonest
        assert_eq!(homework[1].subject, "History"); // 25th
        assert_eq!(homework[2].subject, "Math");    // 28th - latest
    }

    #[test]
    fn test_homework_sorting_past_descending() {
        // Past homework should be sorted by due_date descending (newest first)
        let mut homework = vec![
            Homework {
                id: Some(1),
                subject: "Math".to_string(),
                text: "HW 1".to_string(),
                date: "10.02.2026".to_string(),
                due_date: Some("12.02.2026".to_string()),
                date_sort: Some("2026-02-10".to_string()),
                due_date_sort: Some("2026-02-12".to_string()),
            },
            Homework {
                id: Some(2),
                subject: "English".to_string(),
                text: "HW 2".to_string(),
                date: "05.02.2026".to_string(),
                due_date: Some("07.02.2026".to_string()),
                date_sort: Some("2026-02-05".to_string()),
                due_date_sort: Some("2026-02-07".to_string()),
            },
            Homework {
                id: Some(3),
                subject: "History".to_string(),
                text: "HW 3".to_string(),
                date: "15.02.2026".to_string(),
                due_date: Some("17.02.2026".to_string()),
                date_sort: Some("2026-02-15".to_string()),
                due_date_sort: Some("2026-02-17".to_string()),
            },
        ];

        // Sort descending by due_date (newest first)
        homework.sort_by(|a, b| {
            let a_due = a.due_date_sort.as_deref().unwrap_or("0000-00-00");
            let b_due = b.due_date_sort.as_deref().unwrap_or("0000-00-00");
            b_due.cmp(a_due)
        });

        assert_eq!(homework[0].subject, "History"); // 17th - most recent
        assert_eq!(homework[1].subject, "Math");    // 12th
        assert_eq!(homework[2].subject, "English"); // 7th - oldest
    }
}
