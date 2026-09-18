use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single grade entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeEntry {
    pub value: String,
    pub date: Option<String>,
    pub description: Option<String>,
    pub grade_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grade {
    pub subject: String,
    pub term1_grades: Vec<GradeEntry>,
    pub term2_grades: Vec<GradeEntry>,
    pub term1_final: Option<GradeEntry>,
    pub term2_final: Option<GradeEntry>,
    pub annual: Option<GradeEntry>,
}

/// Raw grade from the /grades endpoint (flat list)
#[derive(Debug, Clone, Deserialize)]
pub struct GradeRaw {
    pub grade: Option<String>,
    pub grade_date: Option<String>,
    pub description: Option<String>,
    pub course_name: Option<String>,
    pub grade_type_name: Option<String>,
    pub type_id: Option<i32>,
    pub term_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradesListResponse {
    pub grades: Option<Vec<GradeRaw>>,
}

// --- Legacy types for /grades/summary endpoint (kept for reference) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeDetail {
    pub grade: Option<String>,
    pub grade_raw: Option<String>,
    pub numerical_value: Option<f64>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TermGrades {
    Map(HashMap<String, GradeDetail>),
    List(Vec<GradeDetail>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseGrades {
    pub target_name: Option<String>,
    pub course_name: Option<String>,
    pub term1: Option<TermGrades>,
    pub term2: Option<TermGrades>,
    pub term1final: Option<TermGrades>,
    pub term2final: Option<TermGrades>,
    pub annual: Option<TermGrades>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradesSummaryResponse {
    pub grades: Option<Vec<CourseGrades>>,
    pub courses: Option<Vec<CourseGrades>>,
}

// Grade type IDs from the API
const TERM_FINAL_TYPE_ID: i32 = 5; // "Срочна" - term final grade
const ANNUAL_TYPE_ID: i32 = 6;     // Annual grade (assumed)

impl Grade {
    /// Build grades from the flat /grades endpoint, grouped by subject
    pub fn from_grades_list(raw_grades: &[GradeRaw]) -> Vec<Grade> {
        // Group by course_name
        let mut by_subject: HashMap<String, Vec<&GradeRaw>> = HashMap::new();
        for g in raw_grades {
            let subject = g.course_name.clone().unwrap_or_else(|| "Unknown".to_string());
            by_subject.entry(subject).or_default().push(g);
        }

        let mut grades: Vec<Grade> = by_subject.into_iter().map(|(subject, entries)| {
            let mut term1_grades = Vec::new();
            let mut term2_grades = Vec::new();
            let mut term1_final = None;
            let mut term2_final = None;
            let mut annual = None;

            for g in entries {
                let value = match &g.grade {
                    Some(v) => v.clone(),
                    None => continue,
                };

                let entry = GradeEntry {
                    value,
                    date: g.grade_date.clone(),
                    description: g.description.clone(),
                    grade_type: g.grade_type_name.clone(),
                };

                let type_id = g.type_id.unwrap_or(0);
                let term_id = g.term_id.unwrap_or(0);

                if type_id == ANNUAL_TYPE_ID {
                    annual = Some(entry);
                } else if type_id == TERM_FINAL_TYPE_ID {
                    match term_id {
                        1 => term1_final = Some(entry),
                        2 => term2_final = Some(entry),
                        _ => {}
                    }
                } else {
                    match term_id {
                        1 => term1_grades.push(entry),
                        2 => term2_grades.push(entry),
                        _ => {}
                    }
                }
            }

            // Sort grades by date (oldest first, newest last)
            // Dates are DD.MM.YYYY, convert to YYYY-MM-DD for sorting
            term1_grades.sort_by(|a, b| date_sort_key(&a.date).cmp(&date_sort_key(&b.date)));
            term2_grades.sort_by(|a, b| date_sort_key(&a.date).cmp(&date_sort_key(&b.date)));

            Grade {
                subject,
                term1_grades,
                term2_grades,
                term1_final,
                term2_final,
                annual,
            }
        }).collect();

        // Sort subjects alphabetically for stable ordering
        grades.sort_by(|a, b| a.subject.cmp(&b.subject));

        grades
    }

    /// Build from the legacy /grades/summary endpoint (fallback)
    pub fn from_course_grades(course: &CourseGrades) -> Self {
        let subject = course.target_name.clone()
            .or_else(|| course.course_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let term1_grades = extract_grades(&course.term1);
        let term2_grades = extract_grades(&course.term2);
        let term1_final = extract_final_grade(&course.term1final);
        let term2_final = extract_final_grade(&course.term2final);
        let annual = extract_final_grade(&course.annual);

        Self {
            subject,
            term1_grades,
            term2_grades,
            term1_final,
            term2_final,
            annual,
        }
    }

    pub fn has_grades(&self) -> bool {
        !self.term1_grades.is_empty()
            || !self.term2_grades.is_empty()
            || self.term1_final.is_some()
            || self.term2_final.is_some()
            || self.annual.is_some()
    }
}

/// Convert DD.MM.YYYY date to sortable YYYY-MM-DD format
fn date_sort_key(date: &Option<String>) -> String {
    match date {
        Some(d) => {
            let parts: Vec<&str> = d.split('.').collect();
            if parts.len() == 3 {
                format!("{}-{}-{}", parts[2], parts[1], parts[0])
            } else {
                d.clone()
            }
        }
        None => String::new(),
    }
}

impl GradeEntry {
    pub fn value_str(&self) -> &str {
        &self.value
    }
}

fn extract_grade_value(detail: &GradeDetail) -> Option<String> {
    if let Some(g) = &detail.grade {
        return Some(g.clone());
    }
    if let Some(g) = &detail.grade_raw {
        return Some(g.clone());
    }
    if let Some(n) = detail.numerical_value {
        return Some(n.to_string());
    }
    None
}

fn extract_grades(term: &Option<TermGrades>) -> Vec<GradeEntry> {
    let mut grades = Vec::new();
    match term {
        Some(TermGrades::Map(map)) => {
            // Sort by key for stable ordering
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            for (_, detail) in entries {
                if let Some(g) = extract_grade_value(detail) {
                    grades.push(GradeEntry {
                        value: g,
                        date: None,
                        description: None,
                        grade_type: None,
                    });
                }
            }
        }
        Some(TermGrades::List(list)) => {
            for detail in list {
                if let Some(g) = extract_grade_value(detail) {
                    grades.push(GradeEntry {
                        value: g,
                        date: None,
                        description: None,
                        grade_type: None,
                    });
                }
            }
        }
        None => {}
    }
    grades
}

fn extract_final_grade(term: &Option<TermGrades>) -> Option<GradeEntry> {
    match term {
        Some(TermGrades::Map(map)) => {
            for detail in map.values() {
                if let Some(g) = extract_grade_value(detail) {
                    return Some(GradeEntry {
                        value: g,
                        date: None,
                        description: None,
                        grade_type: None,
                    });
                }
            }
        }
        Some(TermGrades::List(list)) => {
            for detail in list {
                if let Some(g) = extract_grade_value(detail) {
                    return Some(GradeEntry {
                        value: g,
                        date: None,
                        description: None,
                        grade_type: None,
                    });
                }
            }
        }
        None => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_grades_sorted_by_key() {
        let mut map = HashMap::new();
        map.insert("2026-03-10".to_string(), GradeDetail {
            grade: Some("4".into()), grade_raw: None, numerical_value: None, icon: None,
        });
        map.insert("2026-01-15".to_string(), GradeDetail {
            grade: Some("6".into()), grade_raw: None, numerical_value: None, icon: None,
        });
        map.insert("2026-02-20".to_string(), GradeDetail {
            grade: Some("5".into()), grade_raw: None, numerical_value: None, icon: None,
        });

        let term = Some(TermGrades::Map(map));
        let grades = extract_grades(&term);

        // Should be sorted by key: oldest first, newest last
        let values: Vec<&str> = grades.iter().map(|g| g.value.as_str()).collect();
        assert_eq!(values, vec!["6", "5", "4"]);
    }

    #[test]
    fn test_from_grades_list() {
        let raw = vec![
            GradeRaw {
                grade: Some("6".into()),
                grade_date: Some("15.01.2026".into()),
                description: Some("homework".into()),
                course_name: Some("Math".into()),
                grade_type_name: Some("Домашна работа".into()),
                type_id: Some(3),
                term_id: Some(1),
            },
            GradeRaw {
                grade: Some("5".into()),
                grade_date: Some("20.02.2026".into()),
                description: None,
                course_name: Some("Math".into()),
                grade_type_name: Some("Тест".into()),
                type_id: Some(7),
                term_id: Some(2),
            },
            GradeRaw {
                grade: Some("6".into()),
                grade_date: Some("25.01.2026".into()),
                description: None,
                course_name: Some("Math".into()),
                grade_type_name: Some("Срочна".into()),
                type_id: Some(5), // term final
                term_id: Some(1),
            },
        ];

        let grades = Grade::from_grades_list(&raw);
        assert_eq!(grades.len(), 1);
        let math = &grades[0];
        assert_eq!(math.subject, "Math");
        assert_eq!(math.term1_grades.len(), 1);
        assert_eq!(math.term1_grades[0].value, "6");
        assert_eq!(math.term1_grades[0].description.as_deref(), Some("homework"));
        assert_eq!(math.term2_grades.len(), 1);
        assert_eq!(math.term2_grades[0].value, "5");
        assert!(math.term1_final.is_some());
        assert_eq!(math.term1_final.as_ref().unwrap().value, "6");
    }

    #[test]
    fn test_grades_sorted_by_date() {
        let raw = vec![
            GradeRaw {
                grade: Some("3".into()),
                grade_date: Some("20.02.2026".into()),
                description: None,
                course_name: Some("BG".into()),
                grade_type_name: None,
                type_id: Some(4),
                term_id: Some(1),
            },
            GradeRaw {
                grade: Some("6".into()),
                grade_date: Some("10.01.2026".into()),
                description: None,
                course_name: Some("BG".into()),
                grade_type_name: None,
                type_id: Some(4),
                term_id: Some(1),
            },
            GradeRaw {
                grade: Some("5".into()),
                grade_date: Some("15.01.2026".into()),
                description: None,
                course_name: Some("BG".into()),
                grade_type_name: None,
                type_id: Some(4),
                term_id: Some(1),
            },
        ];

        let grades = Grade::from_grades_list(&raw);
        let values: Vec<&str> = grades[0].term1_grades.iter().map(|g| g.value.as_str()).collect();
        // DD.MM.YYYY dates sorted chronologically: 10.01 < 15.01 < 20.02
        assert_eq!(values, vec!["6", "5", "3"]);
    }

    #[test]
    fn test_date_sort_key() {
        assert_eq!(date_sort_key(&Some("05.03.2026".into())), "2026-03-05");
        assert_eq!(date_sort_key(&Some("15.12.2025".into())), "2025-12-15");
        assert_eq!(date_sort_key(&None), "");
    }
}
