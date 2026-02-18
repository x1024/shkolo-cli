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
