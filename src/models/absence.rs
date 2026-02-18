use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsenceRaw {
    pub id: Option<String>,
    pub date: Option<String>,
    pub school_hour: Option<i32>,
    pub course_name: Option<String>,
    pub course_short_name: Option<String>,
    pub absence_type_id: Option<i32>,
    pub absence_excuse_type_id: Option<i32>,
    pub absence_comment: Option<String>,
    pub created_by: Option<String>,
    pub created_date_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsencesResponse {
    pub absences: Option<Vec<AbsenceRaw>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Absence {
    pub id: String,
    pub date: String,         // DD.MM.YYYY format
    pub date_sort: String,    // YYYY-MM-DD for sorting
    pub hour: i32,
    pub subject: String,
    pub is_excused: bool,
    pub excuse_reason: Option<String>,
    pub created_by: Option<String>,
}

impl Absence {
    pub fn from_raw(raw: &AbsenceRaw) -> Self {
        let date = raw.date.clone().unwrap_or_default();

        // Convert DD.MM.YYYY to YYYY-MM-DD for sorting
        let date_sort = if date.len() >= 10 {
            let parts: Vec<&str> = date.split('.').collect();
            if parts.len() == 3 {
                format!("{}-{}-{}", parts[2], parts[1], parts[0])
            } else {
                date.clone()
            }
        } else {
            date.clone()
        };

        // absence_excuse_type_id: 1 = excused, 0 or null = unexcused
        let is_excused = raw.absence_excuse_type_id.unwrap_or(0) == 1;

        Self {
            id: raw.id.clone().unwrap_or_default(),
            date,
            date_sort,
            hour: raw.school_hour.unwrap_or(0),
            subject: raw.course_short_name.clone()
                .or_else(|| raw.course_name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            is_excused,
            excuse_reason: raw.absence_comment.clone(),
            created_by: raw.created_by.clone(),
        }
    }
}
