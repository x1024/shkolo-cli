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
pub struct UserRole {
    pub role_id: Option<i64>,
    pub role_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolYear {
    pub id: i64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub names: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub years: Option<Vec<SchoolYear>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersAndYearsResponse {
    pub users: Option<Vec<User>>,
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
