use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grade {
    pub subject: String,
    pub term1_grades: Vec<String>,
    pub term2_grades: Vec<String>,
    pub term1_final: Option<String>,
    pub term2_final: Option<String>,
    pub annual: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GradeValue {
    Simple(String),
    Numeric(f64),
    Detailed(GradeDetail),
}

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

impl Grade {
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

fn extract_grades(term: &Option<TermGrades>) -> Vec<String> {
    let mut grades = Vec::new();
    match term {
        Some(TermGrades::Map(map)) => {
            for detail in map.values() {
                if let Some(g) = extract_grade_value(detail) {
                    grades.push(g);
                }
            }
        }
        Some(TermGrades::List(list)) => {
            for detail in list {
                if let Some(g) = extract_grade_value(detail) {
                    grades.push(g);
                }
            }
        }
        None => {}
    }
    grades
}

fn extract_final_grade(term: &Option<TermGrades>) -> Option<String> {
    match term {
        Some(TermGrades::Map(map)) => {
            for detail in map.values() {
                if let Some(g) = extract_grade_value(detail) {
                    return Some(g);
                }
            }
        }
        Some(TermGrades::List(list)) => {
            for detail in list {
                if let Some(g) = extract_grade_value(detail) {
                    return Some(g);
                }
            }
        }
        None => {}
    }
    None
}
