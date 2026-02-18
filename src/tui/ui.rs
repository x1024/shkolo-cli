use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{App, Focus, Tab};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(frame.area());

    draw_tabs(frame, app, chunks[0]);
    draw_content(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.current_tab {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(t.name(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Shkolo "))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(Tab::all().iter().position(|t| *t == app.current_tab).unwrap_or(0));

    frame.render_widget(tabs, area);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),  // Students list
            Constraint::Min(40),     // Main content
        ])
        .split(area);

    draw_students_list(frame, app, chunks[0]);

    match app.current_tab {
        Tab::Overview => draw_overview(frame, app, chunks[1]),
        Tab::Homework => draw_homework(frame, app, chunks[1]),
        Tab::Grades => draw_grades(frame, app, chunks[1]),
        Tab::Schedule => draw_schedule(frame, app, chunks[1]),
        Tab::Notifications => draw_notifications(frame, app, chunks[1]),
    }
}

fn draw_students_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Students;

    let items: Vec<ListItem> = app.students
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let is_selected = i == app.selected_student;
            let style = if is_selected {
                if is_focused {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let prefix = if is_selected { "> " } else { "  " };
            let class_suffix = data.student.class_name
                .as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default();

            ListItem::new(format!("{}{}{}", prefix, data.student.name, class_suffix))
                .style(style)
        })
        .collect();

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Students "));

    frame.render_widget(list, area);
}

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Schedule
            Constraint::Percentage(35),  // Recent homework
            Constraint::Percentage(25),  // Grades summary
        ])
        .split(area);

    draw_overview_schedule(frame, app, chunks[0]);
    draw_overview_homework(frame, app, chunks[1]);
    draw_overview_grades(frame, app, chunks[2]);
}

fn draw_overview_schedule(frame: &mut Frame, app: &App, area: Rect) {
    let current_time = app.current_time;
    let current_minutes = current_time.0 as i32 * 60 + current_time.1 as i32;

    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new("  No classes scheduled for today")]
        } else {
            data.schedule
                .iter()
                .map(|hour| {
                    // Parse times to determine if lesson has passed
                    let (from_h, from_m) = parse_time(&hour.from_time);
                    let (to_h, to_m) = parse_time(&hour.to_time);
                    let from_mins = from_h * 60 + from_m;
                    let to_mins = to_h * 60 + to_m;

                    let is_past = to_mins < current_minutes;
                    let is_current = from_mins <= current_minutes && current_minutes < to_mins;

                    let time = format!("{}-{}", hour.from_time, hour.to_time);

                    let style = if is_current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if is_past {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    let marker = if is_current { " <" } else { "" };

                    let line = format!(
                        "  {}. [{}] {}{}",
                        hour.hour_number, time, hour.subject, marker
                    );

                    ListItem::new(line).style(style)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let time_str = format!("{:02}:{:02}", current_time.0, current_time.1);
    let title = format!(" Today's Schedule ({}) [{}] ", app.current_date, time_str);

    let is_focused = app.focus == Focus::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_overview_homework(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        let recent = data.recent_homework();
        if recent.is_empty() {
            vec![ListItem::new("  No recent homework")]
        } else {
            recent.iter()
                .take(5)
                .map(|hw| {
                    let due_str = hw.due_date
                        .as_ref()
                        .map(|d| format!(" -> {}", d))
                        .unwrap_or_default();

                    let line = format!(
                        "  [{}] {}{}: {}",
                        hw.date, hw.subject, due_str,
                        truncate(&hw.text, 40)
                    );

                    ListItem::new(line)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Recent Homework "));

    frame.render_widget(list, area);
}

fn draw_overview_grades(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        let total = data.total_grades_count();
        let summary = data.recent_grades_summary();

        if summary.is_empty() {
            vec![ListItem::new(format!("  Total grades: {}", total))]
        } else {
            let mut items = vec![
                ListItem::new(format!("  Total grades: {}", total)),
                ListItem::new(""),
            ];

            for (subject, grades) in summary {
                let grades_str = grades.join(", ");
                items.push(ListItem::new(format!("  {}: {}", truncate(subject, 20), grades_str)));
            }

            items
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Grades Summary "));

    frame.render_widget(list, area);
}

fn draw_homework(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        if data.homework.is_empty() {
            vec![ListItem::new("  No homework found")]
        } else {
            data.homework
                .iter()
                .skip(app.list_offset)
                .take(area.height.saturating_sub(2) as usize / 3)
                .map(|hw| {
                    let due_str = hw.due_date
                        .as_ref()
                        .map(|d| format!(" -> Due: {}", d))
                        .unwrap_or_default();

                    let lines = vec![
                        Line::from(Span::styled(
                            format!("  [{}] {}{}", hw.date, hw.subject, due_str),
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Line::from(format!("    {}", hw.text)),
                        Line::from(""),
                    ];

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let age = app.current_student()
        .and_then(|d| d.homework_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" Homework (newest first) [{}] ", age);

    let is_focused = app.focus == Focus::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_grades(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        if data.grades.is_empty() {
            vec![ListItem::new("  No grades found")]
        } else {
            data.grades
                .iter()
                .skip(app.list_offset)
                .take(area.height.saturating_sub(2) as usize / 4)
                .map(|grade| {
                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  {}", grade.subject),
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                    ];

                    if !grade.term1_grades.is_empty() {
                        let grades_str = grade.term1_grades.join(", ");
                        let avg = calculate_average(&grade.term1_grades);
                        let avg_str = avg.map(|a| format!(" (avg: {:.2})", a)).unwrap_or_default();
                        lines.push(Line::from(format!("    Term 1: {}{}", grades_str, avg_str)));
                    }

                    if let Some(ref final_grade) = grade.term1_final {
                        lines.push(Line::from(Span::styled(
                            format!("    Term 1 Final: {}", final_grade),
                            Style::default().fg(Color::Green),
                        )));
                    }

                    if !grade.term2_grades.is_empty() {
                        let grades_str = grade.term2_grades.join(", ");
                        let avg = calculate_average(&grade.term2_grades);
                        let avg_str = avg.map(|a| format!(" (avg: {:.2})", a)).unwrap_or_default();
                        lines.push(Line::from(format!("    Term 2: {}{}", grades_str, avg_str)));
                    }

                    if let Some(ref final_grade) = grade.term2_final {
                        lines.push(Line::from(Span::styled(
                            format!("    Term 2 Final: {}", final_grade),
                            Style::default().fg(Color::Green),
                        )));
                    }

                    if let Some(ref annual) = grade.annual {
                        lines.push(Line::from(Span::styled(
                            format!("    Annual: {}", annual),
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        )));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let age = app.current_student()
        .and_then(|d| d.grades_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" Grades [{}] ", age);

    let is_focused = app.focus == Focus::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_schedule(frame: &mut Frame, app: &App, area: Rect) {
    let current_time = app.current_time;
    let current_minutes = current_time.0 as i32 * 60 + current_time.1 as i32;

    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new("  No schedule for this day")]
        } else {
            data.schedule
                .iter()
                .map(|hour| {
                    let (from_h, from_m) = parse_time(&hour.from_time);
                    let (to_h, to_m) = parse_time(&hour.to_time);
                    let from_mins = from_h * 60 + from_m;
                    let to_mins = to_h * 60 + to_m;

                    let is_past = to_mins < current_minutes;
                    let is_current = from_mins <= current_minutes && current_minutes < to_mins;

                    let time = format!("{}-{}", hour.from_time, hour.to_time);

                    let header_style = if is_current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if is_past {
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    };

                    let detail_style = if is_past {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    let marker = if is_current { " <NOW" } else { "" };

                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  {}. [{}] {}{}", hour.hour_number, time, hour.subject, marker),
                            header_style,
                        )),
                    ];

                    if let Some(ref teacher) = hour.teacher {
                        lines.push(Line::from(Span::styled(
                            format!("     Teacher: {}", teacher),
                            detail_style,
                        )));
                    }

                    if let Some(ref topic) = hour.topic {
                        lines.push(Line::from(Span::styled(
                            format!("     Topic: {}", topic),
                            detail_style,
                        )));
                    }

                    if let Some(ref homework) = hour.homework {
                        lines.push(Line::from(Span::styled(
                            format!("     HW: {}", homework),
                            Style::default().fg(Color::Cyan),
                        )));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let age = app.current_student()
        .and_then(|d| d.schedule_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let time_str = format!("{:02}:{:02}", current_time.0, current_time.1);
    let title = format!(" Schedule for {} [{}] ({}) ", app.current_date, age, time_str);

    let is_focused = app.focus == Focus::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_notifications(frame: &mut Frame, app: &App, area: Rect) {
    let content = if app.notifications.is_empty() {
        vec![ListItem::new("  No notifications")]
    } else {
        app.notifications
            .iter()
            .skip(app.list_offset)
            .take(area.height.saturating_sub(2) as usize / 3)
            .map(|notif| {
                let read_style = if notif.is_read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                };

                let read_marker = if notif.is_read { "" } else { "[NEW] " };

                let mut lines = vec![
                    Line::from(Span::styled(
                        format!("  {}{}", read_marker, notif.title),
                        read_style,
                    )),
                ];

                if let Some(ref body) = notif.body {
                    lines.push(Line::from(format!("    {}", truncate(body, 60))));
                }

                lines.push(Line::from(Span::styled(
                    format!("    {}", notif.date),
                    Style::default().fg(Color::DarkGray),
                )));

                lines.push(Line::from(""));

                ListItem::new(lines)
            })
            .collect()
    };

    let age = app.notifications_age
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let unread_count = app.notifications.iter().filter(|n| !n.is_read).count();
    let title = if unread_count > 0 {
        format!(" Notifications ({} unread) [{}] ", unread_count, age)
    } else {
        format!(" Notifications [{}] ", age)
    };

    let is_focused = app.focus == Focus::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else if app.loading {
        "Loading...".to_string()
    } else {
        "".to_string()
    };

    let refresh_info = app.last_refresh
        .as_ref()
        .map(|t| format!("Last: {}", t))
        .unwrap_or_default();

    let focus_hint = match app.focus {
        Focus::Students => "[Tab]->Content",
        Focus::Content => "[Tab]->Students",
    };

    let help = format!("[R]efresh [Q]uit [</>]Tabs [^v]Select {} [1-5]Student", focus_hint);

    let content = Line::from(vec![
        Span::styled(
            format!(" {} ", help),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::styled(
            status,
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            refresh_info,
            Style::default().fg(Color::Green),
        ),
    ]);

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

fn calculate_average(grades: &[String]) -> Option<f64> {
    let numeric: Vec<f64> = grades
        .iter()
        .filter_map(|g| g.parse().ok())
        .collect();

    if numeric.is_empty() {
        None
    } else {
        Some(numeric.iter().sum::<f64>() / numeric.len() as f64)
    }
}

fn parse_time(time_str: &str) -> (i32, i32) {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 2 {
        let h = parts[0].parse().unwrap_or(0);
        let m = parts[1].parse().unwrap_or(0);
        (h, m)
    } else {
        (0, 0)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    } else {
        s.to_string()
    }
}
