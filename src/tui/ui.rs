use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::i18n::T;
use super::app::{App, Focus, Tab, InputMode, MessageView};
use super::handlers::get_keybindings;

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

    // Draw error overlay if there's an error (loading indicator is now in status bar only)
    if let Some(ref error) = app.error_message {
        draw_error_overlay(frame, error);
    }

    // Draw help overlay if requested
    if app.show_help {
        draw_help_overlay(frame, app);
    }
}

fn draw_error_overlay(frame: &mut Frame, error: &str) {
    let area = frame.area();

    // Fixed width for error box
    let width = (area.width as usize * 2 / 3).max(40).min(area.width as usize - 4) as u16;
    let inner_width = width.saturating_sub(4) as usize;

    // Wrap text to fit within the box
    let wrapped_lines = wrap_text(error, inner_width, "");

    let height = (wrapped_lines.len() as u16 + 4).min(area.height - 4);

    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;

    let error_area = Rect::new(x, y, width, height);

    // Join wrapped lines
    let wrapped_text = wrapped_lines.join("\n");

    let error_text = Paragraph::new(wrapped_text)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .style(Style::default().fg(Color::Red))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(" Error [Press any key to dismiss] ")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));

    frame.render_widget(Clear, error_area);
    frame.render_widget(error_text, error_area);
}

fn draw_help_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let bindings = get_keybindings(app);

    // Calculate dimensions
    let max_key_len = bindings.iter().map(|(k, _)| k.len()).max().unwrap_or(10);
    let max_desc_len = bindings.iter().map(|(_, d)| d.len()).max().unwrap_or(20);
    let content_width = max_key_len + max_desc_len + 5; // key + " : " + desc + padding
    let width = (content_width as u16 + 4).min(area.width - 4).max(40);
    let height = (bindings.len() as u16 + 4).min(area.height - 4);

    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;

    let help_area = Rect::new(x, y, width, height);

    // Build help text with aligned columns
    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("{:>width$}", key, width = max_key_len),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" : "),
                Span::styled(*desc, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    // Get context description
    let context = get_context_description(app);
    let title = format!(" Keyboard Shortcuts ({}) [Press any key] ", context);

    let help_text = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title)
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

    frame.render_widget(Clear, help_area);
    frame.render_widget(help_text, help_area);
}

/// Get a description of the current context for the help title
fn get_context_description(app: &App) -> &'static str {
    if app.input_mode != InputMode::Normal {
        return match app.input_mode {
            InputMode::Reply => "Replying",
            InputMode::ComposeSubject => "Composing Subject",
            InputMode::ComposeBody => "Composing Message",
            InputMode::Normal => "Normal",
        };
    }

    if app.current_tab == Tab::Messages {
        return match app.message_view {
            MessageView::Thread => "Thread View",
            MessageView::Compose => "Select Recipients",
            MessageView::List => "Messages",
        };
    }

    match app.current_tab {
        Tab::Overview => "Overview",
        Tab::Schedule => "Schedule",
        Tab::Homework => "Homework",
        Tab::Grades => "Grades",
        Tab::Absences => "Absences",
        Tab::Feedbacks => "Feedbacks",
        Tab::Notifications => "Notifications",
        Tab::Messages => "Messages",
        Tab::Settings => "Settings",
    }
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.current_tab {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(t.name(lang), style))
        })
        .collect();

    let title = format!(" {} ", T::app_title(lang));
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(Tab::all().iter().position(|t| *t == app.current_tab).unwrap_or(0));

    frame.render_widget(tabs, area);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    // Notifications and Settings are global (not per-student), so show them full-width
    match app.current_tab {
        Tab::Notifications => {
            draw_notifications(frame, app, area);
            return;
        }
        Tab::Settings => {
            draw_settings(frame, app, area);
            return;
        }
        Tab::Messages => {
            draw_messages(frame, app, area);
            return;
        }
        _ => {}
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(app.students_pane_width),  // Resizable students list
            Constraint::Min(40),     // Main content
        ])
        .split(area);

    draw_students_list(frame, app, chunks[0]);

    match app.current_tab {
        Tab::Overview => draw_overview(frame, app, chunks[1]),
        Tab::Homework => draw_homework(frame, app, chunks[1]),
        Tab::Grades => draw_grades(frame, app, chunks[1]),
        Tab::Schedule => draw_schedule(frame, app, chunks[1]),
        Tab::Absences => draw_absences(frame, app, chunks[1]),
        Tab::Feedbacks => draw_feedbacks(frame, app, chunks[1]),
        Tab::Notifications | Tab::Settings | Tab::Messages => unreachable!(), // Handled above
    }
}

fn draw_students_list(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
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

    let title = format!(" {} ", T::students(lang));
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    // First split: schedule vs rest (configurable)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(app.overview_split_percent),
            Constraint::Percentage(100 - app.overview_split_percent),
        ])
        .split(area);

    // Second split: homework and grades
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Homework
            Constraint::Percentage(40),  // Grades summary
        ])
        .split(main_chunks[1]);

    draw_overview_schedule(frame, app, main_chunks[0]);
    draw_overview_homework(frame, app, bottom_chunks[0]);
    draw_overview_grades(frame, app, bottom_chunks[1]);
}

fn draw_overview_schedule(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let current_time = app.current_time;
    let current_minutes = current_time.0 as i32 * 60 + current_time.1 as i32;

    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_schedule(lang)))]
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
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let time_str = format!("{:02}:{:02}", current_time.0, current_time.1);
    let title = format!(" {} ({}) [{}] ", T::today_schedule(lang), app.current_date, time_str);

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
    let lang = app.lang;
    let text_width = area.width.saturating_sub(4) as usize;
    let today = &app.current_date;

    let content = if let Some(data) = app.current_student() {
        if data.homework.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_homework(lang)))]
        } else {
            // Sort homework by due date (soonest first) - same logic as Homework tab
            let mut sorted_homework: Vec<_> = data.homework.iter().collect();
            sorted_homework.sort_by(|a, b| {
                let a_due = a.due_date_sort.as_deref().unwrap_or("9999-99-99");
                let b_due = b.due_date_sort.as_deref().unwrap_or("9999-99-99");
                a_due.cmp(b_due)
            });

            // Determine when school day ends today (from schedule)
            let school_day_end_minutes = data.schedule.iter()
                .map(|h| {
                    let (to_h, to_m) = parse_time(&h.to_time);
                    to_h * 60 + to_m
                })
                .max()
                .unwrap_or(15 * 60); // Default to 15:00 if no schedule

            let current_minutes = app.current_time.0 as i32 * 60 + app.current_time.1 as i32;
            let school_day_over = current_minutes > school_day_end_minutes;

            // Split into future and past based on due date AND school day - same logic as Homework tab
            let (future, mut past): (Vec<_>, Vec<_>) = sorted_homework.into_iter().partition(|hw| {
                match hw.due_date_sort.as_ref() {
                    Some(d) if d > today => true,  // Future date
                    Some(d) if d < today => false, // Past date
                    Some(_) => !school_day_over,    // Today - depends on school day
                    None => true,                   // No due date - treat as future
                }
            });

            // Sort past by descending due date (newest first)
            past.sort_by(|a, b| {
                let a_due = a.due_date_sort.as_deref().unwrap_or("0000-00-00");
                let b_due = b.due_date_sort.as_deref().unwrap_or("0000-00-00");
                b_due.cmp(a_due)
            });

            // Show up to 5 items: prioritize future homework, then fill with past
            let mut items_to_show: Vec<_> = future.into_iter().take(5).collect();
            let remaining = 5 - items_to_show.len();
            if remaining > 0 {
                items_to_show.extend(past.into_iter().take(remaining));
            }

            items_to_show.iter()
                .flat_map(|hw| {
                    // Check if homework is still pending (considering school day end)
                    let is_future = match hw.due_date_sort.as_ref() {
                        Some(d) if d > today => true,
                        Some(d) if d < today => false,
                        Some(_) => !school_day_over, // Today - depends on school day
                        None => true,
                    };

                    let style = if is_future {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let due_str = hw.due_date
                        .as_ref()
                        .map(|d| format!(" -> {}", d))
                        .unwrap_or_default();

                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  [{}] {}{}", hw.date, hw.subject, due_str),
                            style.add_modifier(Modifier::BOLD),
                        )),
                    ];

                    // Wrap the homework text
                    for wrapped_line in wrap_text(&hw.text, text_width, "    ") {
                        lines.push(Line::from(Span::styled(wrapped_line, style)));
                    }

                    vec![ListItem::new(lines)]
                })
                .collect()
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let is_focused = app.focus == Focus::OverviewHomework;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title = format!(" {} ", T::recent_homework(lang));
    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_overview_grades(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let content = if let Some(data) = app.current_student() {
        let total = data.total_grades_count();
        let summary = data.recent_grades_summary();

        if summary.is_empty() {
            vec![ListItem::new(format!("  {}: {}", T::total_grades(lang), total))]
        } else {
            let mut items = vec![
                ListItem::new(Line::from(Span::styled(
                    format!("  {}: {}", T::total_grades(lang), total),
                    Style::default().add_modifier(Modifier::BOLD),
                ))),
            ];

            for (subject, grades) in summary {
                // Calculate average for these grades
                let grade_strings: Vec<String> = grades.iter().map(|s| s.to_string()).collect();
                let avg = calculate_average(&grade_strings);

                let mut spans = vec![
                    Span::raw(format!("  {}: ", truncate(subject, 15))),
                ];

                // Average first (colored)
                if let Some(a) = avg {
                    spans.push(Span::styled(
                        format!("{:.1}", a),
                        Style::default().fg(average_color(a)).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(" <- "));
                }

                // Individual grades (colored)
                for (i, g) in grades.iter().enumerate() {
                    if i > 0 { spans.push(Span::raw(", ")); }
                    spans.push(Span::styled(g.to_string(), Style::default().fg(grade_color(g))));
                }

                items.push(ListItem::new(Line::from(spans)));
            }

            items
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let is_focused = app.focus == Focus::OverviewGrades;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title = format!(" {} ", T::grades_summary(lang));
    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_homework(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let text_width = area.width.saturating_sub(4) as usize; // Account for borders and padding
    let today = &app.current_date;

    let content = if let Some(data) = app.current_student() {
        if data.homework.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_homework(lang)))]
        } else {
            // Determine when school day ends today (from schedule)
            let school_day_end_minutes = data.schedule.iter()
                .map(|h| {
                    let (to_h, to_m) = parse_time(&h.to_time);
                    to_h * 60 + to_m
                })
                .max()
                .unwrap_or(15 * 60); // Default to 15:00 if no schedule

            let current_minutes = app.current_time.0 as i32 * 60 + app.current_time.1 as i32;
            let school_day_over = current_minutes > school_day_end_minutes;

            // Split into future and past based on due date AND school day
            let (mut future, mut past): (Vec<_>, Vec<_>) = data.homework.iter().partition(|hw| {
                match hw.due_date_sort.as_ref() {
                    Some(d) if d > today => true,  // Future date
                    Some(d) if d < today => false, // Past date
                    Some(_) => !school_day_over,    // Today - depends on school day
                    None => true,                   // No due date - treat as future
                }
            });

            // Sort future by due date (soonest first)
            future.sort_by(|a, b| {
                let a_due = a.due_date_sort.as_deref().unwrap_or("9999-99-99");
                let b_due = b.due_date_sort.as_deref().unwrap_or("9999-99-99");
                a_due.cmp(b_due)
            });

            // Sort past by due date (newest/most recent first)
            past.sort_by(|a, b| {
                let a_due = a.due_date_sort.as_deref().unwrap_or("0000-00-00");
                let b_due = b.due_date_sort.as_deref().unwrap_or("0000-00-00");
                b_due.cmp(a_due) // Reversed for newest first
            });

            // Build combined list of items with divider
            #[derive(Clone)]
            enum HomeworkItem<'a> {
                Future(&'a crate::models::Homework),
                Divider,
                Past(&'a crate::models::Homework),
            }

            let mut all_items: Vec<HomeworkItem> = Vec::new();

            // Future items first
            for hw in &future {
                all_items.push(HomeworkItem::Future(hw));
            }

            // Add divider if we have both future and past items
            if !future.is_empty() && !past.is_empty() {
                all_items.push(HomeworkItem::Divider);
            }

            // Past items
            for hw in &past {
                all_items.push(HomeworkItem::Past(hw));
            }

            // Now skip and render
            let mut items = Vec::new();

            for item in all_items.into_iter().skip(app.list_offset) {
                match item {
                    HomeworkItem::Future(hw) => {
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
                    HomeworkItem::Divider => {
                        let divider = format!("  ─────────────── {} ───────────────", T::past_due(lang));
                        items.push(ListItem::new(Line::from(Span::styled(
                            divider,
                            Style::default().fg(Color::DarkGray),
                        ))));
                    }
                    HomeworkItem::Past(hw) => {
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
                }
            }

            items
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let age = app.current_student()
        .and_then(|d| d.homework_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" {} ({}) ", T::homework(lang), age);

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
    let lang = app.lang;
    let content = if let Some(data) = app.current_student() {
        if data.grades.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_grades(lang)))]
        } else {
            data.grades
                .iter()
                .skip(app.list_offset)
                .take(area.height.saturating_sub(2) as usize / 5)
                .map(|grade| {
                    let mut lines = vec![
                        Line::from(Span::styled(
                            format!("  {}", grade.subject),
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                    ];

                    // Term 1: Show average first, then grades
                    if !grade.term1_grades.is_empty() {
                        let avg = calculate_average(&grade.term1_grades);
                        let mut spans = vec![Span::raw(format!("    {}: ", T::term1(lang)))];

                        // Average first (colored)
                        if let Some(a) = avg {
                            spans.push(Span::styled(
                                format!("{:.2}", a),
                                Style::default().fg(average_color(a)).add_modifier(Modifier::BOLD),
                            ));
                            spans.push(Span::raw(" <- "));
                        }

                        // Individual grades (colored)
                        for (i, g) in grade.term1_grades.iter().enumerate() {
                            if i > 0 { spans.push(Span::raw(", ")); }
                            spans.push(Span::styled(g.clone(), Style::default().fg(grade_color(g))));
                        }

                        lines.push(Line::from(spans));
                    }

                    if let Some(ref final_grade) = grade.term1_final {
                        lines.push(Line::from(Span::styled(
                            format!("    {} {}: {}", T::term1(lang), T::final_grade(lang), final_grade),
                            Style::default().fg(grade_color(final_grade)).add_modifier(Modifier::BOLD),
                        )));
                    }

                    // Term 2: Show average first, then grades
                    if !grade.term2_grades.is_empty() {
                        let avg = calculate_average(&grade.term2_grades);
                        let mut spans = vec![Span::raw(format!("    {}: ", T::term2(lang)))];

                        // Average first (colored)
                        if let Some(a) = avg {
                            spans.push(Span::styled(
                                format!("{:.2}", a),
                                Style::default().fg(average_color(a)).add_modifier(Modifier::BOLD),
                            ));
                            spans.push(Span::raw(" <- "));
                        }

                        // Individual grades (colored)
                        for (i, g) in grade.term2_grades.iter().enumerate() {
                            if i > 0 { spans.push(Span::raw(", ")); }
                            spans.push(Span::styled(g.clone(), Style::default().fg(grade_color(g))));
                        }

                        lines.push(Line::from(spans));
                    }

                    if let Some(ref final_grade) = grade.term2_final {
                        lines.push(Line::from(Span::styled(
                            format!("    {} {}: {}", T::term2(lang), T::final_grade(lang), final_grade),
                            Style::default().fg(grade_color(final_grade)).add_modifier(Modifier::BOLD),
                        )));
                    }

                    if let Some(ref annual) = grade.annual {
                        lines.push(Line::from(Span::styled(
                            format!("    {}: {}", T::annual(lang), annual),
                            Style::default().fg(grade_color(annual)).add_modifier(Modifier::BOLD),
                        )));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let age = app.current_student()
        .and_then(|d| d.grades_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" {} ({}) ", T::grades(lang), age);

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
    let lang = app.lang;
    let current_time = app.current_time;
    let current_minutes = current_time.0 as i32 * 60 + current_time.1 as i32;
    let is_today = app.is_schedule_today();

    let content = if let Some(data) = app.current_student() {
        if data.schedule.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_schedule(lang)))]
        } else {
            data.schedule
                .iter()
                .map(|hour| {
                    let (from_h, from_m) = parse_time(&hour.from_time);
                    let (to_h, to_m) = parse_time(&hour.to_time);
                    let from_mins = from_h * 60 + from_m;
                    let to_mins = to_h * 60 + to_m;

                    // Only highlight current/past when viewing today
                    let is_past = is_today && to_mins < current_minutes;
                    let is_current = is_today && from_mins <= current_minutes && current_minutes < to_mins;

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
                            format!("     {}: {}", T::teacher(lang), teacher),
                            detail_style,
                        )));
                    }

                    if let Some(ref topic) = hour.topic {
                        lines.push(Line::from(Span::styled(
                            format!("     {}: {}", T::topic(lang), topic),
                            detail_style,
                        )));
                    }

                    if let Some(ref homework) = hour.homework {
                        lines.push(Line::from(Span::styled(
                            format!("     {}: {}", T::homework(lang), homework),
                            Style::default().fg(Color::Cyan),
                        )));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect()
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let age = app.current_student()
        .and_then(|d| d.schedule_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Format date for display (YYYY-MM-DD -> DD.MM.YYYY)
    let display_date = {
        let parts: Vec<&str> = app.schedule_date.split('-').collect();
        if parts.len() == 3 {
            format!("{}.{}.{}", parts[2], parts[1], parts[0])
        } else {
            app.schedule_date.clone()
        }
    };

    let today_marker = if is_today {
        match lang { crate::i18n::Lang::Bg => " (днес)", crate::i18n::Lang::En => " (today)" }
    } else { "" };

    let time_str = if is_today {
        format!(" [{:02}:{:02}]", current_time.0, current_time.1)
    } else {
        String::new()
    };

    let nav_hint = match lang {
        crate::i18n::Lang::Bg => " [p/n]-ден [t]-днес",
        crate::i18n::Lang::En => " [p/n]-day [t]-today",
    };

    let title = format!(" {} {}{} ({}){}{}  ", T::schedule(lang), display_date, today_marker, age, time_str, nav_hint);

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

fn draw_absences(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;

    let content = if let Some(data) = app.current_student() {
        if data.absences.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_absences(lang)))]
        } else {
            let mut items = Vec::new();

            // Calculate totals
            let total_excused = data.absences.iter().filter(|a| a.is_excused).count();
            let total_unexcused = data.absences.iter().filter(|a| !a.is_excused).count();
            let total = data.absences.len();

            // Overall summary
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}: ", match lang { crate::i18n::Lang::Bg => "Общо", crate::i18n::Lang::En => "Total" }),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", total),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::raw("("),
                Span::styled(format!("{} {}", total_excused, T::excused(lang)), Style::default().fg(Color::Green)),
                Span::raw(", "),
                Span::styled(format!("{} {}", total_unexcused, T::unexcused(lang)), Style::default().fg(Color::Red)),
                Span::raw(")"),
            ])));

            items.push(ListItem::new(""));

            // Per-subject summary
            let mut subject_counts: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
            for absence in &data.absences {
                let entry = subject_counts.entry(absence.subject.clone()).or_insert((0, 0));
                if absence.is_excused {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
            }

            let mut subjects: Vec<_> = subject_counts.into_iter().collect();
            // Stable sort: by total descending, then by subject name for ties
            subjects.sort_by(|a, b| {
                let total_a = a.1.0 + a.1.1;
                let total_b = b.1.0 + b.1.1;
                total_b.cmp(&total_a).then_with(|| a.0.cmp(&b.0))
            });

            for (subject, (excused, unexcused)) in &subjects {
                let total_subj = excused + unexcused;
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{}: ", subject), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", total_subj), Style::default()),
                    Span::raw("("),
                    Span::styled(format!("{}", excused), Style::default().fg(Color::Green)),
                    Span::raw("/"),
                    Span::styled(format!("{}", unexcused), Style::default().fg(Color::Red)),
                    Span::raw(")"),
                ])));
            }

            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(Span::styled(
                "  ─────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ))));
            items.push(ListItem::new(""));

            // Detailed list grouped by date (scrollable)
            let mut current_date = String::new();

            for absence in data.absences.iter().skip(app.list_offset) {
                // Add date header if new date
                if absence.date != current_date {
                    if !current_date.is_empty() {
                        items.push(ListItem::new("")); // Spacer
                    }
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("  {}", absence.date),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ))));
                    current_date = absence.date.clone();
                }

                // Absence entry
                let status_style = if absence.is_excused {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                let status_text = if absence.is_excused {
                    T::excused(lang)
                } else {
                    T::unexcused(lang)
                };

                let hour_label = T::hour_label(lang);

                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(format!("    {} {}: ", hour_label, absence.hour)),
                        Span::styled(absence.subject.clone(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" - "),
                        Span::styled(status_text, status_style),
                    ]),
                ]));

                // Show excuse reason if present
                if let Some(ref reason) = absence.excuse_reason {
                    if !reason.is_empty() {
                        let wrapped = wrap_text(reason, (area.width as usize).saturating_sub(10), "      ");
                        for line in wrapped {
                            items.push(ListItem::new(Line::from(Span::styled(
                                line,
                                Style::default().fg(Color::DarkGray),
                            ))));
                        }
                    }
                }
            }

            items
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let age = app.current_student()
        .and_then(|d| d.absences_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" {} ({}) ", T::absences(lang), age);

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

fn draw_feedbacks(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;

    let content = if let Some(data) = app.current_student() {
        if data.feedbacks.is_empty() {
            vec![ListItem::new(format!("  {}", T::no_feedbacks(lang)))]
        } else {
            let mut items = Vec::new();

            // Summary
            let positive_count = data.feedbacks.iter().filter(|f| f.is_positive).count();
            let negative_count = data.feedbacks.iter().filter(|f| !f.is_positive).count();

            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}: ", match lang { crate::i18n::Lang::Bg => "Общо", crate::i18n::Lang::En => "Total" }),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", data.feedbacks.len()),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::raw("("),
                Span::styled(format!("{} {}", positive_count, T::positive(lang)), Style::default().fg(Color::Green)),
                Span::raw(", "),
                Span::styled(format!("{} {}", negative_count, T::negative(lang)), Style::default().fg(Color::Red)),
                Span::raw(")"),
            ])));

            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(Span::styled(
                "  ─────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ))));
            items.push(ListItem::new(""));

            // List feedbacks
            for feedback in data.feedbacks.iter().skip(app.list_offset) {
                let emoji = feedback.emoji();
                let style = if feedback.is_positive {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                // Badge name with emoji and date
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::raw(emoji.clone()),
                    Span::raw(" "),
                    Span::styled(feedback.badge_name.clone(), style.add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(feedback.date.clone(), Style::default().fg(Color::DarkGray)),
                ])));

                // Subject and teacher
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(feedback.subject.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(feedback.teacher.clone(), Style::default().fg(Color::DarkGray)),
                ])));

                // Comment if present
                if let Some(ref comment) = feedback.comment {
                    if !comment.is_empty() {
                        items.push(ListItem::new(Line::from(vec![
                            Span::raw("     "),
                            Span::styled(format!("\"{}\"", comment), Style::default().fg(Color::Gray)),
                        ])));
                    }
                }

                items.push(ListItem::new(""));
            }

            items
        }
    } else {
        vec![ListItem::new(format!("  {}", T::no_student(lang)))]
    };

    let age = app.current_student()
        .and_then(|d| d.feedbacks_age.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let title = format!(" {} ({}) ", T::feedbacks(lang), age);

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

fn draw_messages(frame: &mut Frame, app: &App, area: Rect) {
    // Check if we're viewing a thread, compose, or the list
    match app.message_view {
        MessageView::Thread => draw_message_thread(frame, app, area),
        MessageView::List => draw_message_list(frame, app, area),
        MessageView::Compose => draw_compose(frame, app, area),
    }
}

fn draw_message_list(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let text_width = area.width.saturating_sub(4) as usize;

    let content = if app.messages.is_empty() {
        vec![ListItem::new(format!("  {}", T::no_messages(lang)))]
    } else {
        app.messages
            .iter()
            .enumerate()
            .skip(app.list_offset)
            .map(|(idx, msg)| {
                let is_selected = idx == app.list_offset;
                let style = if msg.is_unread {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let unread_marker = if msg.is_unread { T::new_marker(lang) } else { "" };
                let selected_marker = if is_selected { "▸ " } else { "  " };

                let mut lines = Vec::new();

                // Subject line with unread marker
                let subject_text = format!("{}{}{}", selected_marker, unread_marker, msg.subject);
                for wrapped_line in wrap_text(&subject_text, text_width, "  ") {
                    lines.push(Line::from(Span::styled(wrapped_line, style)));
                }

                // Last message preview
                let preview = msg.preview(text_width.saturating_sub(6));
                if !preview.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", preview),
                        Style::default().fg(Color::Gray),
                    )));
                }

                // Sender and time
                let sender_info = format!(
                    "    {} · {} {} · {}",
                    msg.last_sender,
                    msg.participant_count,
                    T::participants(lang),
                    msg.display_time()
                );
                lines.push(Line::from(Span::styled(
                    sender_info,
                    Style::default().fg(Color::DarkGray),
                )));

                lines.push(Line::from(""));

                ListItem::new(lines)
            })
            .collect()
    };

    let age = app.messages_age
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let unread_count = app.messages.iter().filter(|m| m.is_unread).count();
    let enter_hint = match lang {
        crate::i18n::Lang::Bg => "[Enter]-отвори",
        crate::i18n::Lang::En => "[Enter]-open",
    };
    let title = if unread_count > 0 {
        format!(" {} ({} {}) ({}) {} ", T::messages(lang), unread_count, T::unread(lang), age, enter_hint)
    } else {
        format!(" {} ({}) {} ", T::messages(lang), age, enter_hint)
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

fn draw_message_thread(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let text_width = area.width.saturating_sub(4) as usize;

    // If in input mode, split the area
    let (messages_area, input_area) = if app.input_mode == InputMode::Reply {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Get current thread subject
    let subject = app.messages
        .iter()
        .find(|m| Some(m.id) == app.selected_thread_id)
        .map(|m| m.subject.clone())
        .unwrap_or_else(|| "Thread".to_string());

    // Build message content
    let content: Vec<ListItem> = if app.thread_messages.is_empty() {
        vec![ListItem::new(format!("  {}", T::loading(lang)))]
    } else {
        app.thread_messages
            .iter()
            .skip(app.thread_offset)
            .map(|msg| {
                let mut lines = Vec::new();

                // Sender and date
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", msg.sender_name),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        msg.date.clone(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));

                // Message body
                for wrapped_line in wrap_text(&msg.body, text_width, "    ") {
                    lines.push(Line::from(Span::raw(wrapped_line)));
                }

                lines.push(Line::from(""));

                ListItem::new(lines)
            })
            .collect()
    };

    let reply_hint = match lang {
        crate::i18n::Lang::Bg => "[r]-отговор [Esc]-назад",
        crate::i18n::Lang::En => "[r]-reply [Esc]-back",
    };

    let title = format!(" {} {} ", subject, reply_hint);

    let list = List::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(title));

    frame.render_widget(list, messages_area);

    // Draw input area if in reply mode
    if let Some(input_rect) = input_area {
        let input_title = match lang {
            crate::i18n::Lang::Bg => " Отговор (Enter - изпрати, Esc - отказ) ",
            crate::i18n::Lang::En => " Reply (Enter - send, Esc - cancel) ",
        };

        // Show cursor position
        let cursor_x = input_rect.x + 1 + app.input_cursor as u16;
        let cursor_y = input_rect.y + 1;

        let input = Paragraph::new(app.input_buffer.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(input_title));

        frame.render_widget(input, input_rect);

        // Set cursor position
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_compose(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;

    // Split area: recipients list (left), compose fields (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Recipients
            Constraint::Percentage(60),  // Subject & Body
        ])
        .split(area);

    // Draw recipients list
    let recipients_title = match lang {
        crate::i18n::Lang::Bg => " Получатели (Space - избери, S - напред) ",
        crate::i18n::Lang::En => " Recipients (Space - select, S - next) ",
    };

    let recipient_items: Vec<ListItem> = if app.recipients.is_empty() {
        vec![ListItem::new(format!("  {}", T::loading(lang)))]
    } else {
        app.recipients
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let is_selected = app.selected_recipients.contains(&r.id);
                let is_current = idx == app.list_offset;

                let marker = if is_selected { "[✓] " } else { "[ ] " };
                let cursor = if is_current { "▸ " } else { "  " };

                let style = if is_selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                ListItem::new(format!("{}{}{}", cursor, marker, r.name)).style(style)
            })
            .collect()
    };

    let recipients_list = List::new(recipient_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if app.input_mode == InputMode::Normal {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
            .title(recipients_title));

    frame.render_widget(recipients_list, chunks[0]);

    // Draw compose fields
    let compose_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Subject
            Constraint::Min(5),     // Body
        ])
        .split(chunks[1]);

    // Subject field
    let subject_title = match lang {
        crate::i18n::Lang::Bg => " Тема ",
        crate::i18n::Lang::En => " Subject ",
    };

    let subject_text = if app.input_mode == InputMode::ComposeSubject {
        app.input_buffer.as_str()
    } else {
        app.compose_subject.as_str()
    };

    let subject_style = if app.input_mode == InputMode::ComposeSubject {
        Style::default().fg(Color::Yellow)
    } else if !app.compose_subject.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let subject = Paragraph::new(subject_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(subject_style)
            .title(subject_title));

    frame.render_widget(subject, compose_chunks[0]);

    // Body field
    let body_title = match lang {
        crate::i18n::Lang::Bg => " Съобщение (Enter - изпрати) ",
        crate::i18n::Lang::En => " Message (Enter - send) ",
    };

    let body_text = if app.input_mode == InputMode::ComposeBody {
        app.input_buffer.as_str()
    } else {
        ""
    };

    let body_style = if app.input_mode == InputMode::ComposeBody {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let body = Paragraph::new(body_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(body_style)
            .title(body_title));

    frame.render_widget(body, compose_chunks[1]);

    // Set cursor position if in input mode
    if app.input_mode == InputMode::ComposeSubject {
        let cursor_x = compose_chunks[0].x + 1 + app.input_cursor as u16;
        let cursor_y = compose_chunks[0].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else if app.input_mode == InputMode::ComposeBody {
        let cursor_x = compose_chunks[1].x + 1 + app.input_cursor as u16;
        let cursor_y = compose_chunks[1].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_notifications(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let text_width = area.width.saturating_sub(4) as usize;

    let content = if app.notifications.is_empty() {
        vec![ListItem::new(format!("  {}", T::no_notifications(lang)))]
    } else {
        app.notifications
            .iter()
            .enumerate()
            .skip(app.list_offset)
            .map(|(idx, notif)| {
                let is_selected = idx == app.list_offset;
                let read_style = if notif.is_read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                };

                let read_marker = if notif.is_read { "" } else { T::new_marker(lang) };
                let selected_marker = if is_selected { "▸ " } else { "  " };

                let mut lines = Vec::new();

                // Wrap title with selection marker
                let title_text = format!("{}{}{}", selected_marker, read_marker, notif.title);
                for (i, wrapped_line) in wrap_text(&title_text, text_width, "  ").into_iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(Span::styled(wrapped_line, read_style)));
                    } else {
                        lines.push(Line::from(Span::styled(wrapped_line, read_style)));
                    }
                }

                // Wrap body if present
                if let Some(ref body) = notif.body {
                    for wrapped_line in wrap_text(body, text_width, "      ") {
                        lines.push(Line::from(Span::styled(wrapped_line, Style::default().fg(Color::Gray))));
                    }
                }

                // Pupil name and date on same line
                let pupil_info = notif.pupil_names.as_ref()
                    .map(|p| format!("[{}] ", p))
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    Span::styled(format!("      {}", pupil_info), Style::default().fg(Color::Cyan)),
                    Span::styled(notif.date.clone(), Style::default().fg(Color::DarkGray)),
                ]));

                lines.push(Line::from(""));

                ListItem::new(lines)
            })
            .collect()
    };

    let age = app.notifications_age
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let unread_count = app.notifications.iter().filter(|n| !n.is_read).count();
    let enter_hint = match lang {
        crate::i18n::Lang::Bg => "[Enter]-отвори",
        crate::i18n::Lang::En => "[Enter]-open",
    };
    let title = if unread_count > 0 {
        format!(" {} ({} {}) ({}) {} ", T::notifications(lang), unread_count, T::unread(lang), age, enter_hint)
    } else {
        format!(" {} ({}) {} ", T::notifications(lang), age, enter_hint)
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

fn draw_settings(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;

    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                format!("  {} ", T::account(lang)),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ])),
        ListItem::new(""),
    ];

    // Show current user or login options
    if let Some(ref name) = app.user_name {
        items.push(ListItem::new(Line::from(vec![
            Span::raw(format!("  {}: ", T::logged_in_as(lang))),
            Span::styled(name.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])));
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  [L] {}", T::logout(lang)),
            Style::default().fg(Color::Yellow),
        ))));
    } else {
        // Not logged in - show login options
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", T::login(lang)),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(Color::Yellow)),
            Span::raw(T::login_password(lang)),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
            Span::raw(T::login_google(lang)),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  [3] ", Style::default().fg(Color::Yellow)),
            Span::raw(T::import_token(lang)),
        ])));
        items.push(ListItem::new(Line::from(Span::styled(
            format!("      {}", T::import_token_desc(lang)),
            Style::default().fg(Color::DarkGray),
        ))));
    }

    items.push(ListItem::new(""));
    items.push(ListItem::new(Line::from(Span::raw("  ─────────────────────────────"))));
    items.push(ListItem::new(""));

    // Language toggle
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  [G] ", Style::default().fg(Color::Yellow)),
        Span::raw("Език / Language: "),
        Span::styled(
            match lang {
                crate::i18n::Lang::Bg => "Български",
                crate::i18n::Lang::En => "English",
            },
            Style::default().fg(Color::Cyan),
        ),
    ])));

    let title = format!(" {} ", T::settings(lang));

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title));

    frame.render_widget(list, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;

    // Show spinner when loading
    let status = if app.loading {
        let spinner_idx = app.tick % SPINNER_FRAMES.len();
        let spinner = SPINNER_FRAMES[spinner_idx];
        let msg = app.status_message.as_deref().unwrap_or(T::loading(lang));
        format!("{} {}", spinner, msg)
    } else if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "".to_string()
    };

    let refresh_info = app.last_refresh
        .as_ref()
        .map(|t| format!("Last: {}", t))
        .unwrap_or_default();

    let user_info = app.user_name
        .as_ref()
        .map(|n| format!("[{}]", n))
        .unwrap_or_default();

    let focus_hint = match app.focus {
        Focus::Students => format!("[Tab]->{}", T::schedule(lang)),
        Focus::OverviewSchedule => format!("[Tab]->{}", T::homework(lang)),
        Focus::OverviewHomework => format!("[Tab]->{}", T::grades(lang)),
        Focus::OverviewGrades => format!("[Tab]->{}", T::students(lang)),
        Focus::Content => format!("[Tab]->{}", T::students(lang)),
    };

    let help = format!("{} {} {} {} {} [1-5] [-/+]", T::help_refresh(lang), T::help_quit(lang), T::help_tabs(lang), T::help_select(lang), focus_hint);

    // Left side: help text and status
    let left_content = Line::from(vec![
        Span::styled(
            format!(" {} ", help),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::styled(
            status,
            Style::default().fg(Color::Yellow),
        ),
    ]);

    // Right side: refresh info and user name
    let right_content = Line::from(vec![
        Span::styled(
            refresh_info,
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} ", user_info),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    // Split status bar into left and right
    let inner_area = Block::default().borders(Borders::ALL).inner(area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(right_content.width() as u16 + 1),
        ])
        .split(inner_area);

    // Render border
    frame.render_widget(Block::default().borders(Borders::ALL), area);

    // Render left content
    let left_para = Paragraph::new(left_content);
    frame.render_widget(left_para, chunks[0]);

    // Render right content (right-aligned)
    let right_para = Paragraph::new(right_content).alignment(Alignment::Right);
    frame.render_widget(right_para, chunks[1]);
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

/// Get color for a grade value (Bulgarian grading: 2-6 scale)
/// 6 = Excellent (green), 5 = Very Good (cyan), 4 = Good (yellow)
/// 3 = Satisfactory (magenta), 2 = Poor (red)
fn grade_color(grade: &str) -> Color {
    match grade.chars().next() {
        Some('6') => Color::Green,
        Some('5') => Color::Cyan,
        Some('4') => Color::Yellow,
        Some('3') => Color::Magenta,
        Some('2') => Color::Red,
        _ => Color::White,
    }
}

/// Get color for an average grade value
fn average_color(avg: f64) -> Color {
    if avg >= 5.5 { Color::Green }
    else if avg >= 4.5 { Color::Cyan }
    else if avg >= 3.5 { Color::Yellow }
    else if avg >= 2.5 { Color::Magenta }
    else { Color::Red }
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
