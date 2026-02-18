use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleHour {
    pub hour_number: i32,
    pub from_time: String,
    pub to_time: String,
    pub subject: String,
    pub teacher: Option<String>,
    pub topic: Option<String>,
    pub homework: Option<String>,
    pub room: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleHourRaw {
    pub school_hour: Option<i32>,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
    pub course_name: Option<String>,
    pub teacher_name: Option<String>,
    pub topic: Option<String>,
    pub homework_text: Option<String>,
    pub room_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    #[serde(rename = "scheduleHours")]
    pub schedule_hours: Option<Vec<ScheduleHourRaw>>,
    pub data: Option<Vec<ScheduleHourRaw>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub event_type: Option<String>,
    pub is_test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRaw {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<i32>,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub data: Option<Vec<EventRaw>>,
    pub invitations: Option<Vec<EventRaw>>,
}

impl ScheduleHour {
    pub fn from_raw(raw: &ScheduleHourRaw) -> Self {
        Self {
            hour_number: raw.school_hour.unwrap_or(0),
            from_time: raw.from_time.clone().unwrap_or_default(),
            to_time: raw.to_time.clone().unwrap_or_default(),
            subject: raw.course_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            teacher: raw.teacher_name.clone(),
            topic: raw.topic.clone(),
            homework: raw.homework_text.clone(),
            room: raw.room_name.clone(),
        }
    }
}

impl Event {
    pub fn from_raw(raw: &EventRaw) -> Self {
        // Event types 12-15 are test/homework related
        let is_test = matches!(raw.event_type, Some(12) | Some(13) | Some(14) | Some(15));

        Self {
            id: raw.id,
            title: raw.title.clone()
                .or_else(|| raw.name.clone())
                .unwrap_or_else(|| "Untitled".to_string()),
            description: raw.description.clone(),
            start_date: raw.start_date.clone().unwrap_or_default(),
            end_date: raw.end_date.clone(),
            event_type: raw.type_name.clone(),
            is_test,
        }
    }
}
