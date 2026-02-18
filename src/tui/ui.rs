use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{App, Tab};

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
    }
}

fn draw_students_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.students
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let style = if i == app.selected_student {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if i == app.selected_student { "> " } else { "  " };
            let class_suffix = data.student.class_name
                .as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default();

            ListItem::new(format!("{}{}{}", prefix, data.student.name, class_suffix))
                .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Students "));

    frame.render_widget(list, area);
}

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Today's schedule
    draw_today_schedule(frame, app, chunks[0]);

    // Upcoming events/tests
    draw_upcoming_events(frame, app, chunks[1]);
}

fn draw_today_schedule(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new("  No classes scheduled for today")]
        } else {
            data.schedule
                .iter()
                .map(|hour| {
                    let time = format!("{}-{}", hour.from_time, hour.to_time);
                    let main_line = format!(
                        "  {}. [{}] {}",
                        hour.hour_number, time, hour.subject
                    );

                    let mut lines = vec![Line::from(main_line)];

                    if let Some(ref teacher) = hour.teacher {
                        lines.push(Line::from(format!("     Teacher: {}", teacher)));
                    }

                    if let Some(ref homework) = hour.homework {
                        lines.push(Line::from(Span::styled(
                            format!("     HW: {}", homework),
                            Style::default().fg(Color::Cyan),
                        )));
                    }

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let title = format!(" Today's Schedule ({}) ", app.current_date);
    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn draw_upcoming_events(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        if data.events.is_empty() {
            vec![ListItem::new("  No upcoming events")]
        } else {
            data.events
                .iter()
                .take(10)
                .map(|event| {
                    let style = if event.is_test {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let prefix = if event.is_test { "[TEST] " } else { "" };

                    ListItem::new(format!(
                        "  {} {} - {}",
                        event.start_date, prefix, event.title
                    ))
                    .style(style)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Upcoming Events "));

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
    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(title));

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
    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn draw_schedule(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new("  No schedule for this day")]
        } else {
            data.schedule
                .iter()
                .map(|hour| {
                    let time = format!("{}-{}", hour.from_time, hour.to_time);

                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  {}. [{}] {}", hour.hour_number, time, hour.subject),
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                    ];

                    if let Some(ref teacher) = hour.teacher {
                        lines.push(Line::from(format!("     Teacher: {}", teacher)));
                    }

                    if let Some(ref topic) = hour.topic {
                        lines.push(Line::from(format!("     Topic: {}", topic)));
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

    let title = format!(" Schedule for {} [{}] ", app.current_date, age);
    let list = List::new(content)
        .block(Block::default().borders(Borders::ALL).title(title));

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

    let help = "[R]efresh [Q]uit [j/k]Navigate [Tab]Switch [1-3]Student";

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
