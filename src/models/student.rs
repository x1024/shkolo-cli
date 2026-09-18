use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub name: String,
    pub class_name: Option<String>,
    pub school_name: Option<String>,
    /// School context the pupil belongs to (parent user id + school year).
    /// Per-pupil endpoints only work while the server is on this context.
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub school_year: Option<i64>,
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

/// A selectable school context: one parent "user" identity in one school for one
/// school year. Parents with children in several schools get one context per school.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchoolContext {
    pub user_id: i64,
    pub school_year: i64,
    pub school_name: String,
    pub user_name: Option<String>,
    /// Marked as the currently selected user+year by the server
    pub is_selected: bool,
}

impl SchoolContext {
    /// Build contexts from usersAndYears entries: newest year first, then by school name.
    pub fn from_entries(entries: &[UserYearEntry]) -> Vec<SchoolContext> {
        let mut contexts: Vec<SchoolContext> = entries
            .iter()
            .filter_map(|e| {
                let user_id = e.id?;
                let school_year = e.school_year?;
                Some(SchoolContext {
                    user_id,
                    school_year,
                    school_name: e.school_name.clone().unwrap_or_else(|| format!("#{}", user_id)),
                    user_name: e.user_name.clone(),
                    is_selected: e.is_selected == Some(true),
                })
            })
            .collect();

        // Stable ordering: newest year first, then school name, then user id
        contexts.sort_by(|a, b| {
            b.school_year.cmp(&a.school_year)
                .then_with(|| a.school_name.cmp(&b.school_name))
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        contexts.dedup_by(|a, b| a.user_id == b.user_id && a.school_year == b.school_year);
        contexts
    }

    /// The latest school year among the contexts (the "current" year)
    pub fn current_year(contexts: &[SchoolContext]) -> Option<i64> {
        contexts.iter().map(|c| c.school_year).max()
    }

    pub fn is_current(&self, current_year: Option<i64>) -> bool {
        Some(self.school_year) == current_year
    }

    pub fn key(&self) -> (i64, i64) {
        (self.user_id, self.school_year)
    }

    /// School year id 2025 means the 2025/26 school year
    pub fn year_label(&self) -> String {
        Self::year_label_for(self.school_year)
    }

    pub fn year_label_for(school_year: i64) -> String {
        format!("{}/{:02}", school_year, (school_year + 1) % 100)
    }
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
            user_id: None,
            school_year: None,
        }
    }

    /// Tag the pupil with the school context it was fetched under
    pub fn with_context(mut self, ctx: &SchoolContext) -> Self {
        self.user_id = Some(ctx.user_id);
        self.school_year = Some(ctx.school_year);
        if self.school_name.is_none() {
            self.school_name = Some(ctx.school_name.clone());
        }
        self
    }

    pub fn context(&self) -> Option<(i64, i64)> {
        Some((self.user_id?, self.school_year?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i64, year: i64, school: &str, selected: bool) -> UserYearEntry {
        UserYearEntry {
            id: Some(id),
            user_name: Some("Parent".into()),
            school_year: Some(year),
            school_name: Some(school.into()),
            is_selected: Some(selected),
            roles: None,
            roles_ids: None,
            region_id: None,
            notification_count: None,
        }
    }

    #[test]
    fn test_from_entries_sorts_and_dedups() {
        let entries = vec![
            entry(3000892, 2024, "СУ \"Димчо Дебелянов\"", false),
            entry(3562478, 2025, "ОУ \"Петко Р. Славейков\"", true),
            entry(3562478, 2025, "ОУ \"Петко Р. Славейков\"", true), // duplicate
            entry(4000001, 2025, "Б школо", false),
            entry(3562478, 2023, "ОУ \"Петко Р. Славейков\"", false),
        ];
        let contexts = SchoolContext::from_entries(&entries);

        assert_eq!(contexts.len(), 4);
        // Newest year first, then by school name
        assert_eq!(contexts[0].key(), (4000001, 2025));
        assert_eq!(contexts[1].key(), (3562478, 2025));
        assert!(contexts[1].is_selected);
        assert_eq!(contexts[2].key(), (3000892, 2024));
        assert_eq!(contexts[3].key(), (3562478, 2023));

        assert_eq!(SchoolContext::current_year(&contexts), Some(2025));
        assert!(contexts[0].is_current(Some(2025)));
        assert!(!contexts[2].is_current(Some(2025)));
    }

    #[test]
    fn test_from_entries_skips_incomplete() {
        let mut broken = entry(1, 2025, "X", false);
        broken.id = None;
        let contexts = SchoolContext::from_entries(&[broken, entry(2, 2025, "Y", false)]);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].user_id, 2);
    }

    #[test]
    fn test_labels_and_keys() {
        let ctx = &SchoolContext::from_entries(&[entry(3562478, 2025, "S", false)])[0];
        assert_eq!(ctx.year_label(), "2025/26");
        assert_eq!(SchoolContext::year_label_for(2099), "2099/00");
    }
}
