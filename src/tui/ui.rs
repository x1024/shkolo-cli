use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{App, Focus, Tab};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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

    // Draw loading overlay if loading
    if app.loading {
        draw_loading_overlay(frame, app);
    }
}

fn draw_loading_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate center position for the loading box
    let width = 30u16;
    let height = 5u16;
    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;

    let loading_area = Rect::new(x, y, width, height);

    // Get spinner frame based on tick counter
    let spinner_idx = app.tick % SPINNER_FRAMES.len();
    let spinner = SPINNER_FRAMES[spinner_idx];

    let message = app.status_message.as_deref().unwrap_or("Loading...");
    let text = format!("{} {}", spinner, message);

    let loading_text = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Loading ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

    // Clear the area first, then render the loading box
    frame.render_widget(Clear, loading_area);
    frame.render_widget(loading_text, loading_area);
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

    let is_focused = app.focus == Focus::OverviewSchedule;
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
    let text_width = area.width.saturating_sub(4) as usize;

    let content = if let Some(data) = app.current_student() {
        let recent = data.recent_homework();
        if recent.is_empty() {
            vec![ListItem::new("  No recent homework")]
        } else {
            recent.iter()
                .take(5)
                .flat_map(|hw| {
                    let due_str = hw.due_date
                        .as_ref()
                        .map(|d| format!(" -> {}", d))
                        .unwrap_or_default();

                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  [{}] {}{}", hw.date, hw.subject, due_str),
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                    ];

                    // Wrap the homework text
                    for wrapped_line in wrap_text(&hw.text, text_width, "    ") {
                        lines.push(Line::from(wrapped_line));
                    }

                    vec![ListItem::new(lines)]
                })
                .collect()
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let is_focused = app.focus == Focus::OverviewHomework;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Recent Homework "));

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

    let is_focused = app.focus == Focus::OverviewGrades;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Grades Summary "));

    frame.render_widget(list, area);
}

fn draw_homework(frame: &mut Frame, app: &App, area: Rect) {
    let text_width = area.width.saturating_sub(4) as usize; // Account for borders and padding

    let content = if let Some(data) = app.current_student() {
        if data.homework.is_empty() {
            vec![ListItem::new("  No homework found")]
        } else {
            // Sort homework by due date (soonest first)
            let mut sorted_homework: Vec<_> = data.homework.iter().collect();
            sorted_homework.sort_by(|a, b| {
                let a_due = a.due_date_sort.as_deref().unwrap_or("9999-99-99");
                let b_due = b.due_date_sort.as_deref().unwrap_or("9999-99-99");
                a_due.cmp(b_due)
            });

            // Split into future (due today or later) and past (due before today)
            let today = &app.current_date;
            let (future, past): (Vec<_>, Vec<_>) = sorted_homework.into_iter().partition(|hw| {
                hw.due_date_sort.as_ref()
                    .map(|d| d >= today)
                    .unwrap_or(true)
            });

            let mut items = Vec::new();

            // Future homework first (upcoming, due today or later)
            for hw in future.iter().skip(app.list_offset) {
                let due_str = hw.due_date
                    .as_ref()
                    .map(|d| format!(" -> Due: {}", d))
                    .unwrap_or_default();

                let mut lines = vec![
                    Line::from(Span::styled(
                        format!("  [{}] {}{}", hw.date, hw.subject, due_str),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )),
                ];

                // Wrap the homework text
                for wrapped_line in wrap_text(&hw.text, text_width, "    ") {
                    lines.push(Line::from(Span::styled(
                        wrapped_line,
                        Style::default().fg(Color::Green),
                    )));
                }
                lines.push(Line::from(""));

                items.push(ListItem::new(lines));
            }

            // Add divider if we have both future and past items
            if !future.is_empty() && !past.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  ─────────────── Past due ───────────────",
                    Style::default().fg(Color::DarkGray),
                ))));
            }

            // Past homework (overdue)
            for hw in past.iter() {
                let due_str = hw.due_date
                    .as_ref()
                    .map(|d| format!(" -> Due: {}", d))
                    .unwrap_or_default();

                let mut lines = vec![
                    Line::from(Span::styled(
                        format!("  [{}] {}{}", hw.date, hw.subject, due_str),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                    )),
                ];

                // Wrap the homework text
                for wrapped_line in wrap_text(&hw.text, text_width, "    ") {
                    lines.push(Line::from(Span::styled(
                        wrapped_line,
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                lines.push(Line::from(""));

                items.push(ListItem::new(lines));
            }

            items
        }
    } else {
        vec![ListItem::new("  No student selected")]
    };

    let age = app.current_student()
        .and_then(|d| d.homework_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" Homework (by due date) [{}] ", age);

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
    let text_width = area.width.saturating_sub(4) as usize;

    let content = if app.notifications.is_empty() {
        vec![ListItem::new("  No notifications")]
    } else {
        app.notifications
            .iter()
            .skip(app.list_offset)
            .map(|notif| {
                let read_style = if notif.is_read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                };

                let read_marker = if notif.is_read { "" } else { "[NEW] " };

                let mut lines = Vec::new();

                // Wrap title
                let title_text = format!("{}{}", read_marker, notif.title);
                for wrapped_line in wrap_text(&title_text, text_width, "  ") {
                    lines.push(Line::from(Span::styled(wrapped_line, read_style)));
                }

                // Wrap body if present
                if let Some(ref body) = notif.body {
                    for wrapped_line in wrap_text(body, text_width, "    ") {
                        lines.push(Line::from(wrapped_line));
                    }
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
        Focus::Students => "[Tab]->Schedule",
        Focus::OverviewSchedule => "[Tab]->Homework",
        Focus::OverviewHomework => "[Tab]->Grades",
        Focus::OverviewGrades => "[Tab]->Students",
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

/// Wrap text to fit within a given width, returning multiple lines
fn wrap_text(s: &str, width: usize, indent: &str) -> Vec<String> {
    if width == 0 || s.is_empty() {
        return vec![format!("{}{}", indent, s)];
    }

    let effective_width = width.saturating_sub(indent.chars().count());
    if effective_width == 0 {
        return vec![format!("{}{}", indent, s)];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;

    for word in s.split_whitespace() {
        let word_len = word.chars().count();

        if current_len == 0 {
            // First word on line
            current_line = word.to_string();
            current_len = word_len;
        } else if current_len + 1 + word_len <= effective_width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
            current_len += 1 + word_len;
        } else {
            // Word doesn't fit, start new line
            lines.push(format!("{}{}", indent, current_line));
            current_line = word.to_string();
            current_len = word_len;
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        lines.push(format!("{}{}", indent, current_line));
    }

    if lines.is_empty() {
        lines.push(indent.to_string());
    }

    lines
}
