---
name: shkolo
description: CLI tool for Shkolo.bg - Bulgarian school management system. Access student grades, homework, schedules, absences, feedbacks, messages, and notifications.
version: 0.1.0
author: x
metadata:
  openclaw:
    emoji: "📚"
    bins:
      - shkolo
    install:
      cargo: shkolo
    os:
      - darwin
      - linux
    tags:
      - education
      - school
      - grades
      - homework
      - bulgaria
---

# Shkolo CLI

A command-line interface for [Shkolo.bg](https://shkolo.bg) - Bulgaria's most popular school management system for parents and students.

## Features

- **Student Management**: View all registered students/children
- **Grades**: Get detailed grade information by subject and term
- **Homework**: Track assignments and due dates
- **Schedule**: View daily class schedules
- **Absences**: Monitor student absences (excused/unexcused)
- **Feedbacks**: View teacher badges, remarks, and feedbacks
- **Notifications**: Stay updated with school notifications
- **Messages**: Read and send messages to teachers

## Authentication

Before using, authenticate using one of these methods:

```bash
# Import token from iOS app (if installed on Mac)
shkolo import-token

# Login with username/password
shkolo login

# Check authentication status
shkolo status
```

## Commands

### JSON Mode (for automation)

```bash
# List all students
shkolo json students

# Get homework for a student
shkolo json homework [student_name_or_index]

# Get grades
shkolo json grades [student_name_or_index]

# Get today's schedule
shkolo json schedule [student_name_or_index] [--date YYYY-MM-DD]

# Get absences
shkolo json absences [student_name_or_index]

# Get feedbacks (teacher remarks)
shkolo json feedbacks [student_name_or_index]

# Get notifications; each has `url`, the app's deep link to what it is about
shkolo json notifications

# Get one event (a test, a parents' meeting): name, type, description, start and end.
# The id is the first number in a notification's `events/<id>/invitations/<id>` link
shkolo json event <event_id> [--user-id <parent user id of its school>]

# Get messages
shkolo json messages

# Save a message's attachments (ids from `shkolo json thread <id>`: message id, files[].id),
# under their real names, to ~/Downloads/Shkolo or --out; the thread's school is selected itself
shkolo json download <thread_id> <message_id> [--file <file_id>] [--out <dir>]

# Get complete summary
shkolo json summary

# List schools (one parent "user" per school) for the current school year
shkolo json schools [--all-years]

# Children in several schools are listed together; per-child commands select the
# right school automatically. Manual switch of the server-side active school:
shkolo json switch-school --user-id <id> [--year <school_year_id>]
```

### Interactive TUI

```bash
shkolo tui
```

#### TUI Navigation

| Key | Action |
|-----|--------|
| `←` `→` `h` `l` | Switch tabs |
| `↑` `↓` `j` `k` | Navigate lists |
| `Tab` | Toggle focus (students/content) |
| `1-5` | Quick select student |
| `Enter` | Open/activate item |
| `r` | Refresh data |
| `R` | Force refresh all |
| `c` | Compose new message (Messages tab) |
| `p` `n` | Previous/Next day (Schedule tab) |
| `t` | Go to today (Schedule tab) |
| `G` | Toggle language (BG/EN) |
| `-` `+` | Resize panes |
| `<` `>` | Resize overview split |
| `q` `Esc` | Quit/Back |

### Cache Management

```bash
# View cache info
shkolo cache

# Clear data cache (keeps token)
shkolo cache --clear

# Clear everything including token
shkolo cache --clear-all

# Force refresh all data
shkolo cache --refresh
```

## Options

| Option | Description |
|--------|-------------|
| `-r, --refresh` | Force refresh data from API |
| `--no-cache` | Bypass cache entirely |
| `--cache-ttl <seconds>` | Set cache TTL (default: 3600) |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SHKOLO_CACHE_TTL` | Default cache TTL in seconds |

## Output Formats

For JSON commands, use `--format`:
- `pretty` (default): Formatted JSON
- `compact`: Minified JSON

## Examples

### Get today's homework for all students

```bash
shkolo json homework
```

### Check grades for a specific student

```bash
shkolo json grades "Maria"
```

### View tomorrow's schedule

```bash
shkolo json schedule --date 2026-02-20
```

### Force refresh and show absences

```bash
shkolo -r json absences
```

## Caching

All data is cached to reduce API calls:
- Default TTL: 1 hour (3600 seconds)
- Cache location: `~/.shkolo/cache/`
- Token stored separately in `~/.shkolo/cache/token.json`

## Language Support

- Bulgarian (BG) - Default
- English (EN)

Toggle with `G` key in TUI or set via API language header.

## Building from Source

```bash
git clone https://github.com/your-username/shkolo
cd shkolo
cargo build --release
```

## License

MIT
