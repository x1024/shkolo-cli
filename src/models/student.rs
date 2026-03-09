use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub name: String,
    pub class_name: Option<String>,
    pub school_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPupil {
    pub target_id: Option<i64>,
    pub target_name: Option<String>,
    pub target_photo: Option<String>,
    pub class_year_id: Option<i64>,
    pub class_year_name: Option<String>,
    pub school_id: Option<i64>,
    pub school_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PupilsResponse {
    #[serde(rename = "childPupils")]
    pub child_pupils: Option<HashMap<String, ChildPupil>>,
    pub pupils: Option<Vec<ChildPupil>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolYear {
    pub id: i64,
    pub name: Option<String>,
}

/// Each entry from /v1/auth/usersAndYears is a user+year combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserYearEntry {
    pub id: Option<i64>,
    #[serde(alias = "user_name")]
    pub user_name: Option<String>,
    #[serde(rename = "schoolYear")]
    pub school_year: Option<i64>,
    pub school_name: Option<String>,
    #[serde(rename = "isSelected")]
    pub is_selected: Option<bool>,
    // Accept roles as any JSON value (string or array)
    #[serde(default)]
    pub roles: Option<serde_json::Value>,
    pub roles_ids: Option<String>,
    pub region_id: Option<i64>,
    #[serde(rename = "notificationCount")]
    pub notification_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersAndYearsResponse {
    pub users: Option<Vec<UserYearEntry>>,
}

impl Student {
    pub fn from_child_pupil(id: &str, pupil: &ChildPupil) -> Self {
        Self {
            id: id.parse().unwrap_or(pupil.target_id.unwrap_or(0)),
            name: pupil.target_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            class_name: pupil.class_year_name.clone(),
            school_name: pupil.school_name.clone(),
        }
    }
}
