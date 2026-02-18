#!/usr/bin/env python3
"""
Shkolo CLI - Command-line client for Shkolo API
"""

import argparse
import json
import os
import sys
from datetime import datetime, timedelta
from getpass import getpass
from pathlib import Path

import requests

# Configuration
API_BASE_URL = "https://api.shkolo.bg"
CONFIG_DIR = Path.home() / ".shkolo"
TOKEN_FILE = CONFIG_DIR / "token.json"
USER_AGENT = "Shkolo-CLI/1.0"
IOS_APP_STORAGE = Path.home() / "Library/Containers/DD1CC5D9-F40E-415C-8E47-094321279222/Data/Library/Application Support/com.shkolo.mobileapp/RCTAsyncLocalStorage_V1/manifest.json"


class ShkoloClient:
    def __init__(self):
        self.token = None
        self.school_year = None
        self.user_data = None
        self.load_token()

    def load_token(self):
        """Load saved token from file."""
        if TOKEN_FILE.exists():
            try:
                with open(TOKEN_FILE) as f:
                    data = json.load(f)
                    self.token = data.get("token")
                    self.school_year = data.get("school_year")
                    self.user_data = data.get("user_data")
            except (json.JSONDecodeError, IOError):
                pass

    def save_token(self):
        """Save token to file."""
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        with open(TOKEN_FILE, "w") as f:
            json.dump({
                "token": self.token,
                "school_year": self.school_year,
                "user_data": self.user_data
            }, f, indent=2)
        os.chmod(TOKEN_FILE, 0o600)

    def clear_token(self):
        """Remove saved token."""
        if TOKEN_FILE.exists():
            TOKEN_FILE.unlink()
        self.token = None
        self.school_year = None
        self.user_data = None

    def get_headers(self, authorized=True):
        """Get request headers."""
        headers = {
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": USER_AGENT,
            "language": "bg"
        }
        if authorized and self.token:
            headers["Authorization"] = f"Bearer {self.token}"
        if self.school_year:
            headers["School-Year"] = str(self.school_year)
        return headers

    def request(self, method, endpoint, data=None, authorized=True):
        """Make an API request."""
        url = f"{API_BASE_URL}{endpoint}"
        headers = self.get_headers(authorized)

        try:
            if method == "GET":
                response = requests.get(url, headers=headers, timeout=30)
            elif method == "POST":
                response = requests.post(url, headers=headers, json=data, timeout=30)
            elif method == "PUT":
                response = requests.put(url, headers=headers, json=data, timeout=30)
            elif method == "DELETE":
                response = requests.delete(url, headers=headers, timeout=30)
            else:
                raise ValueError(f"Unknown method: {method}")

            if response.status_code == 401:
                print("Error: Session expired. Please login again.", file=sys.stderr)
                self.clear_token()
                sys.exit(1)

            return response.json(), response.status_code
        except requests.RequestException as e:
            print(f"Error: Network request failed: {e}", file=sys.stderr)
            sys.exit(1)

    def login(self, username, password):
        """Login to Shkolo."""
        data = {
            "username": username,
            "password": password
        }

        response, status = self.request("POST", "/v1/auth/login", data, authorized=False)

        if status != 200:
            return False, response.get("message", "Login failed")

        self.token = response.get("token")
        if not self.token:
            return False, "No token received"

        # Get users and years to select school year
        users_response, _ = self.request("GET", "/v1/auth/usersAndYears")
        self.user_data = users_response

        # Auto-select first available school year
        if users_response and "users" in users_response:
            for user in users_response["users"]:
                if "years" in user and user["years"]:
                    # Get the most recent year
                    years = sorted(user["years"], key=lambda x: x.get("id", 0), reverse=True)
                    if years:
                        self.school_year = years[0].get("id")
                        break

        self.save_token()
        return True, "Login successful"

    def logout(self):
        """Logout from Shkolo."""
        if self.token:
            self.request("POST", "/v1/auth/logout")
        self.clear_token()

    def is_authenticated(self):
        """Check if user is authenticated."""
        return self.token is not None

    def get_class_years(self):
        """Get all class years."""
        response, status = self.request("GET", "/v1/diary/classYears")
        if status != 200:
            return None
        return response

    def get_homework_list(self, class_year_id):
        """Get homework list for a class year."""
        response, status = self.request("GET", f"/v1/diary/homeworks/list/{class_year_id}")
        if status != 200:
            return None
        return response

    def get_homework_courses(self, class_year_id=None, pupil_id=None):
        """Get homework courses."""
        params = []
        if class_year_id:
            params.append(f"classYearId={class_year_id}")
        if pupil_id:
            params.append(f"pupilId={pupil_id}")

        query = "?" + "&".join(params) if params else ""
        response, status = self.request("GET", f"/v1/diary/homeworks/courses{query}")
        if status != 200:
            return None
        return response

    def get_schedule_hours(self, date=None):
        """Get schedule hours for a date."""
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")
        response, status = self.request("GET", f"/v1/diary/scheduleHours?date={date}")
        if status != 200:
            return None
        return response

    def get_pupils(self):
        """Get pupils (for parents)."""
        response, status = self.request("GET", "/v1/diary/pupils")
        if status != 200:
            return None
        return response

    def get_pupil_schedule(self, pupil_id, date=None):
        """Get schedule for a specific pupil."""
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")
        response, status = self.request("GET", f"/v1/diary/pupils/{pupil_id}/scheduleHours?date={date}")
        if status != 200:
            return None
        return response

    def get_tasks(self):
        """Get assigned tasks (homework)."""
        response, status = self.request("GET", "/v1/tasks/assigned")
        if status != 200:
            return None
        return response

    def get_my_tasks(self):
        """Get user's tasks."""
        response, status = self.request("GET", "/v1/tasks/my-tasks")
        if status != 200:
            return None
        return response

    def get_events(self, school_calendar=False):
        """Get events."""
        endpoint = "/v1/events"
        if school_calendar:
            endpoint += "?is_school_calendar=1"
        response, status = self.request("GET", endpoint)
        if status != 200:
            return None
        return response

    def get_grades(self, pupil_id):
        """Get grades for a pupil."""
        response, status = self.request("GET", f"/v1/diary/pupils/{pupil_id}/grades/summary")
        if status != 200:
            return None
        return response

    def get_absences(self, pupil_id):
        """Get absences for a pupil."""
        response, status = self.request("GET", f"/v1/diary/pupils/{pupil_id}/absences/summary")
        if status != 200:
            return None
        return response

    def get_notifications(self, page=1):
        """Get notifications."""
        response, status = self.request("GET", f"/v1/notifications?page={page}")
        if status != 200:
            return None
        return response


def format_date(date_str):
    """Format date string for display."""
    if not date_str:
        return "N/A"
    try:
        dt = datetime.fromisoformat(date_str.replace("Z", "+00:00"))
        return dt.strftime("%d.%m.%Y %H:%M")
    except (ValueError, AttributeError):
        return date_str


def cmd_login(client, args):
    """Handle login command."""
    username = args.username or input("Username: ")
    password = args.password or getpass("Password: ")

    success, message = client.login(username, password)
    if success:
        print(f"Logged in successfully!")
        if client.user_data and "users" in client.user_data:
            for user in client.user_data["users"]:
                print(f"  User: {user.get('names', 'Unknown')}")
                if "roles" in user:
                    roles = [r.get("role_name", "") for r in user["roles"]]
                    print(f"  Roles: {', '.join(roles)}")
    else:
        print(f"Login failed: {message}", file=sys.stderr)
        sys.exit(1)


def cmd_logout(client, args):
    """Handle logout command."""
    client.logout()
    print("Logged out successfully!")


def cmd_import_token(client, args):
    """Import token from iOS Shkolo app."""
    if not IOS_APP_STORAGE.exists():
        print("Error: Shkolo iOS app data not found.", file=sys.stderr)
        print("Make sure the Shkolo app is installed and you've logged in.", file=sys.stderr)
        sys.exit(1)

    try:
        with open(IOS_APP_STORAGE) as f:
            data = json.load(f)

        token = data.get("@ShkoloStore:Token")
        user_id = data.get("@ShkoloStore:CurrentUserId")
        user_name = data.get("@ShkoloStore:CurrentUserNames")
        roles = data.get("@ShkoloStore:CurrentUserRoles")

        if not token:
            print("Error: No token found in app storage.", file=sys.stderr)
            sys.exit(1)

        client.token = token
        client.user_data = {
            "users": [{
                "id": user_id,
                "names": user_name,
                "roles": [{"role_id": roles}]
            }]
        }
        client.save_token()

        print("Token imported successfully!")
        print(f"User: {user_name}")
        print(f"User ID: {user_id}")
        print(f"Role ID: {roles}")

    except (json.JSONDecodeError, IOError) as e:
        print(f"Error reading app data: {e}", file=sys.stderr)
        sys.exit(1)


def cmd_status(client, args):
    """Show authentication status."""
    if client.is_authenticated():
        print("Status: Authenticated")
        if client.user_data and "users" in client.user_data:
            for user in client.user_data["users"]:
                print(f"User: {user.get('names', 'Unknown')}")
        if client.school_year:
            print(f"School Year ID: {client.school_year}")
    else:
        print("Status: Not authenticated")
        print("Run 'shkolo-cli login' to authenticate")


def cmd_homework(client, args):
    """Get homework."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    print("=" * 60)
    print("SHKOLO - Homework")
    print("=" * 60)
    print()

    # Get pupils (for parent accounts)
    response, status = client.request("GET", "/v1/diary/pupils")
    if status != 200:
        print("Could not fetch pupils data")
        return

    pupils_data = response
    child_pupils = pupils_data.get("childPupils", {})

    if not child_pupils:
        print("No children found. This might be a student account.")
        cmd_homework_student(client, args)
        return

    for pupil_id, pupil in child_pupils.items():
        name = pupil.get("target_name", "Unknown")

        print(f"👤 {name}")
        print("=" * 40)

        # Get homework courses (grouped by subject)
        response, status = client.request("GET", f"/v1/diary/homeworks/courses?pupilId={pupil_id}")
        if status != 200:
            print("   Could not fetch homework data.")
            continue

        courses = response.get("courses", [])
        hw_counts = response.get("cycGroupHomeworksCount", {})

        # Collect all homework from all courses
        all_homework = []

        for course in courses:
            cyc_group_id = course.get("cyc_group_id")
            course_name = course.get("course_short_name", course.get("course_name", "Unknown"))
            hw_count = hw_counts.get(str(cyc_group_id), 0)

            if hw_count == 0:
                continue

            # Fetch homework for this course
            hw_response, hw_status = client.request("GET", f"/v1/diary/homeworks/list/{cyc_group_id}")
            if hw_status == 200:
                for hw in hw_response.get("homeworks", []):
                    all_homework.append({
                        "date": hw.get("shi_date", "N/A"),
                        "date_sort": hw.get("shi_date_for_sort", ""),
                        "due": hw.get("homework_due_date", ""),
                        "text": hw.get("homework_text", ""),
                        "subject": course_name
                    })

        if all_homework:
            # Sort by date, newest first
            sorted_hw = sorted(all_homework, key=lambda x: x["date_sort"], reverse=True)

            # Show last 20 homework items
            for hw in sorted_hw[:20]:
                due_str = f" → Due: {hw['due']}" if hw.get('due') else ""
                print(f"\n   [{hw['date']}] {hw['subject']}{due_str}")
                print(f"   📝 {hw['text']}")

            total = len(all_homework)
            if total > 20:
                print(f"\n   ... and {total - 20} more homework entries")
        else:
            print("\n   No homework found.")

        print()
        print("-" * 60)
        print()

    print("✅ Done")


def cmd_homework_student(client, args):
    """Get homework for student account."""
    # Try getting schedule for current user
    today = datetime.now().strftime("%Y-%m-%d")
    response, status = client.request("GET", f"/v1/diary/scheduleHours?date={today}")

    if status == 200:
        hours = response.get("scheduleHours", response.get("data", []))
        if hours:
            print(f"\n📅 Today's Schedule:")
            for hour in hours:
                hw = hour.get("homework_text")
                print(f"   {hour.get('school_hour', '?')}. {hour.get('course_name', 'N/A')}")
                if hw:
                    print(f"      📝 HOMEWORK: {hw}")

    # Get assigned tasks
    response, status = client.request("GET", "/v1/tasks/assigned")
    if status == 200:
        tasks = response.get("assigned", response.get("data", []))
        if tasks:
            print(f"\n📋 Assigned Tasks:")
            for task in tasks[:10]:
                print(f"   • {task.get('title', task.get('name', 'Untitled'))}")
                if task.get("deadline"):
                    print(f"     Due: {task['deadline']}")


def cmd_homework_legacy(client, args):
    """Legacy homework function."""
    # Also try getting from pupils (for parent accounts)
    pupils = client.get_pupils()
    if pupils:
        items = pupils if isinstance(pupils, list) else pupils.get("data", [])

        for pupil in items[:3]:
            pupil_id = pupil.get("id")
            pupil_name = pupil.get("names", pupil.get("name", f"Pupil {pupil_id}"))

            # Get schedule to find homework
            today = datetime.now()
            for day_offset in range(7):  # Check next 7 days
                date = (today + timedelta(days=day_offset)).strftime("%Y-%m-%d")
                schedule = client.get_pupil_schedule(pupil_id, date)

                if schedule:
                    hours = schedule if isinstance(schedule, list) else schedule.get("data", schedule.get("hours", []))

                    for hour in hours:
                        if hour.get("homework"):
                            hw = hour["homework"]
                            if day_offset == 0:
                                print(f"=== Today's Homework for {pupil_name} ===")
                            else:
                                print(f"=== Homework for {pupil_name} ({date}) ===")
                            print(f"Subject: {hour.get('course_name', 'N/A')}")
                            print(f"Homework: {hw.get('description', hw.get('text', hw))}")
                            print("-" * 40)


def cmd_schedule(client, args):
    """Get schedule."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    date = args.date or datetime.now().strftime("%Y-%m-%d")

    print("=" * 60)
    print(f"SHKOLO - Schedule for {date}")
    print("=" * 60)
    print()

    # Get pupils (for parent accounts)
    response, status = client.request("GET", "/v1/diary/pupils")
    if status != 200:
        print("Could not fetch data")
        return

    child_pupils = response.get("childPupils", {})

    if not child_pupils:
        # Try as student account
        response, status = client.request("GET", f"/v1/diary/scheduleHours?date={date}")
        if status == 200:
            hours = response.get("scheduleHours", [])
            if hours:
                print("📅 My Schedule:\n")
                for hour in sorted(hours, key=lambda x: x.get("school_hour", 0)):
                    time_str = f"{hour.get('from_time', '?')}-{hour.get('to_time', '?')}"
                    print(f"   {hour.get('school_hour', '?')}. [{time_str}] {hour.get('course_name', 'N/A')}")
                    print(f"      Teacher: {hour.get('teacher_name', 'N/A')}")
                    if hour.get("homework_text"):
                        print(f"      📝 Homework: {hour['homework_text']}")
        return

    for pupil_id, pupil in child_pupils.items():
        name = pupil.get("target_name", "Unknown")

        print(f"👤 {name}")
        print("-" * 40)

        response, status = client.request("GET", f"/v1/diary/pupils/{pupil_id}/scheduleHours?date={date}")

        if status == 200:
            hours = response.get("scheduleHours", [])
            if hours:
                for hour in sorted(hours, key=lambda x: x.get("school_hour", 0)):
                    time_str = f"{hour.get('from_time', '?')}-{hour.get('to_time', '?')}"
                    print(f"   {hour.get('school_hour', '?')}. [{time_str}] {hour.get('course_name', 'N/A')}")
                    print(f"      Teacher: {hour.get('teacher_name', 'N/A')}")
                    if hour.get("topic"):
                        print(f"      Topic: {hour['topic']}")
                    if hour.get("homework_text"):
                        print(f"      📝 Homework: {hour['homework_text']}")
            else:
                print("   (No classes scheduled)")
        print()


def cmd_grades(client, args):
    """Get grades."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    print("=" * 60)
    print("SHKOLO - Grades Summary")
    print("=" * 60)
    print()

    # Get pupils (for parent accounts)
    response, status = client.request("GET", "/v1/diary/pupils")
    if status != 200:
        print("Could not fetch data")
        return

    child_pupils = response.get("childPupils", {})

    if not child_pupils:
        print("No children found (student accounts not yet supported for grades)")
        return

    for pupil_id, pupil in child_pupils.items():
        name = pupil.get("target_name", "Unknown")

        print(f"👤 {name}")
        print("=" * 40)

        # Get grades summary
        response, status = client.request("GET", f"/v1/diary/pupils/{pupil_id}/grades/summary")

        if status == 200 and response:
            grades_data = response.get("grades", response.get("courses", []))
            if grades_data:
                print()
                for course in grades_data:
                    course_name = course.get("target_name", course.get("course_name", "Unknown"))

                    # Extract grades from term1 and term2
                    term1_grades = []
                    term2_grades = []

                    term1 = course.get("term1", {})
                    term2 = course.get("term2", {})

                    # Icon to emoji mapping for junior grades
                    icon_map = {
                        "starO": "⭐",
                        "star": "⭐",
                        "heartO": "❤️",
                        "heart": "❤️",
                        "smileO": "😊",
                        "smile": "😊",
                        "mehO": "😐",
                        "meh": "😐",
                        "frownO": "😟",
                        "frown": "😟",
                    }

                    def extract_grade(grade_info):
                        """Extract grade value, handling both numeric and icon-based systems."""
                        if not isinstance(grade_info, dict):
                            return str(grade_info) if grade_info else None
                        # Try numeric grade first
                        g = grade_info.get("grade") or grade_info.get("grade_raw")
                        if g:
                            return str(g)
                        # For junior grades, use numerical_value (the actual grade number)
                        num = grade_info.get("numerical_value")
                        if num:
                            return str(num)
                        return None

                    # term1/term2 can be dict or list
                    if isinstance(term1, dict):
                        for grade_id, grade_info in term1.items():
                            g = extract_grade(grade_info)
                            if g:
                                term1_grades.append(g)
                    elif isinstance(term1, list):
                        for grade_info in term1:
                            g = extract_grade(grade_info)
                            if g:
                                term1_grades.append(g)

                    if isinstance(term2, dict):
                        for grade_id, grade_info in term2.items():
                            g = extract_grade(grade_info)
                            if g:
                                term2_grades.append(g)
                    elif isinstance(term2, list):
                        for grade_info in term2:
                            g = extract_grade(grade_info)
                            if g:
                                term2_grades.append(g)

                    # Get term final grades - can be dict or list
                    term1_final = course.get("term1final", {})
                    term2_final = course.get("term2final", {})
                    annual = course.get("annual", {})

                    def extract_final_grade(final_data):
                        """Extract grade value from final grade data."""
                        if not final_data:
                            return None
                        if isinstance(final_data, dict):
                            for key, val in final_data.items():
                                if isinstance(val, dict):
                                    return val.get("grade")
                                return val
                        elif isinstance(final_data, list):
                            for item in final_data:
                                if isinstance(item, dict):
                                    return item.get("grade")
                                return item
                        return str(final_data)

                    t1_final = extract_final_grade(term1_final)
                    t2_final = extract_final_grade(term2_final)
                    ann_final = extract_final_grade(annual)

                    # Only show if there are any grades
                    if term1_grades or term2_grades or t1_final or t2_final or ann_final:
                        print(f"   📚 {course_name}")
                        if term1_grades:
                            # Check if numeric grades (for averaging)
                            numeric = [float(g) for g in term1_grades if g.replace('.','').isdigit()]
                            if numeric:
                                avg = sum(numeric) / len(numeric)
                                print(f"      Term 1: {', '.join(term1_grades)} (avg: {avg:.2f})")
                            else:
                                print(f"      Term 1: {' '.join(term1_grades)}")
                        if t1_final:
                            print(f"      Term 1 Final: {t1_final}")
                        if term2_grades:
                            numeric = [float(g) for g in term2_grades if g.replace('.','').isdigit()]
                            if numeric:
                                avg = sum(numeric) / len(numeric)
                                print(f"      Term 2: {', '.join(term2_grades)} (avg: {avg:.2f})")
                            else:
                                print(f"      Term 2: {' '.join(term2_grades)}")
                        if t2_final:
                            print(f"      Term 2 Final: {t2_final}")
                        if ann_final:
                            print(f"      Annual: {ann_final}")
            else:
                print("   No grades found")
        else:
            print("   Could not fetch grades")

        print()
        print("-" * 60)
        print()

    # Legacy fallback code below
    return
    pupils_legacy = client.get_pupils()
    if pupils_legacy:
        items = pupils_legacy if isinstance(pupils_legacy, list) else pupils_legacy.get("data", [])

        for pupil in items:
            pupil_id = pupil.get("id")
            pupil_name = pupil.get("names", pupil.get("name", f"Pupil {pupil_id}"))

            grades = client.get_grades(pupil_id)
            if grades:
                print(f"=== Grades for {pupil_name} ===\n")

                grade_items = grades if isinstance(grades, list) else grades.get("data", grades.get("grades", grades.get("courses", [])))

                if isinstance(grade_items, list):
                    for item in grade_items:
                        course = item.get("course_name", item.get("discipline", item.get("name", "N/A")))
                        avg = item.get("average", item.get("avg", ""))
                        term1 = item.get("term1", item.get("first_term", ""))
                        term2 = item.get("term2", item.get("second_term", ""))
                        annual = item.get("annual", item.get("year", ""))

                        print(f"{course}:")
                        if avg:
                            print(f"  Average: {avg}")
                        if term1:
                            print(f"  Term 1: {term1}")
                        if term2:
                            print(f"  Term 2: {term2}")
                        if annual:
                            print(f"  Annual: {annual}")

                        # Show individual grades if available
                        if item.get("grades"):
                            grade_list = [str(g.get("grade", g)) for g in item["grades"][:10]]
                            print(f"  Grades: {', '.join(grade_list)}")
                elif isinstance(grade_items, dict):
                    print(json.dumps(grade_items, indent=2, ensure_ascii=False))
                print()


def cmd_notifications(client, args):
    """Get notifications."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    notifications = client.get_notifications()
    if notifications:
        items = notifications if isinstance(notifications, list) else notifications.get("data", [])

        if items:
            print("=== Notifications ===\n")
            for notif in items[:20]:
                title = notif.get("title", notif.get("subject", "No title"))
                body = notif.get("body", notif.get("message", ""))
                date = format_date(notif.get("created_at", notif.get("date")))
                read = "Read" if notif.get("is_read", notif.get("read")) else "Unread"

                print(f"[{read}] {title}")
                if body:
                    print(f"  {body[:100]}...")
                print(f"  Date: {date}")
                print("-" * 40)
        else:
            print("No notifications")
    else:
        print("Could not fetch notifications")


def cmd_events(client, args):
    """Get events."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    events = client.get_events(school_calendar=args.calendar)
    if events:
        items = events if isinstance(events, list) else events.get("data", [])

        if items:
            print("=== Events ===\n")
            for event in items[:20]:
                title = event.get("title", event.get("name", "Untitled"))
                date = format_date(event.get("start_date", event.get("date")))
                event_type = event.get("type_name", event.get("type", ""))

                print(f"{title}")
                print(f"  Date: {date}")
                if event_type:
                    print(f"  Type: {event_type}")
                if event.get("description"):
                    print(f"  {event['description'][:100]}")
                print("-" * 40)
        else:
            print("No events")
    else:
        print("Could not fetch events")


def cmd_raw(client, args):
    """Make a raw API request."""
    if not client.is_authenticated():
        print("Error: Not authenticated. Run 'shkolo-cli login' first.", file=sys.stderr)
        sys.exit(1)

    method = args.method.upper()
    endpoint = args.endpoint if args.endpoint.startswith("/") else f"/{args.endpoint}"

    data = None
    if args.data:
        try:
            data = json.loads(args.data)
        except json.JSONDecodeError:
            print("Error: Invalid JSON data", file=sys.stderr)
            sys.exit(1)

    response, status = client.request(method, endpoint, data)

    print(f"Status: {status}")
    print(json.dumps(response, indent=2, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser(
        description="Shkolo CLI - Command-line client for Shkolo API",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  shkolo-cli import-token        # Import token from iOS app (recommended)
  shkolo-cli login               # Login with username/password
  shkolo-cli homework            # Get homework and schedule
  shkolo-cli schedule            # Get today's schedule
  shkolo-cli schedule --date 2024-02-20
  shkolo-cli grades
  shkolo-cli raw GET /v1/diary/pupils
        """
    )

    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Import token command
    subparsers.add_parser("import-token", help="Import token from iOS Shkolo app")

    # Login command
    login_parser = subparsers.add_parser("login", help="Login to Shkolo with credentials")
    login_parser.add_argument("-u", "--username", help="Username/email")
    login_parser.add_argument("-p", "--password", help="Password (not recommended, use prompt)")

    # Logout command
    subparsers.add_parser("logout", help="Logout from Shkolo")

    # Status command
    subparsers.add_parser("status", help="Show authentication status")

    # Homework command
    subparsers.add_parser("homework", help="Get homework assignments")

    # Schedule command
    schedule_parser = subparsers.add_parser("schedule", help="Get schedule")
    schedule_parser.add_argument("-d", "--date", help="Date (YYYY-MM-DD)")

    # Grades command
    subparsers.add_parser("grades", help="Get grades")

    # Notifications command
    subparsers.add_parser("notifications", help="Get notifications")

    # Events command
    events_parser = subparsers.add_parser("events", help="Get events")
    events_parser.add_argument("-c", "--calendar", action="store_true", help="Show school calendar")

    # Raw API request
    raw_parser = subparsers.add_parser("raw", help="Make a raw API request")
    raw_parser.add_argument("method", choices=["GET", "POST", "PUT", "DELETE"], help="HTTP method")
    raw_parser.add_argument("endpoint", help="API endpoint (e.g., /v1/diary/pupils)")
    raw_parser.add_argument("-d", "--data", help="JSON data for POST/PUT")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(0)

    client = ShkoloClient()

    commands = {
        "import-token": cmd_import_token,
        "login": cmd_login,
        "logout": cmd_logout,
        "status": cmd_status,
        "homework": cmd_homework,
        "schedule": cmd_schedule,
        "grades": cmd_grades,
        "notifications": cmd_notifications,
        "events": cmd_events,
        "raw": cmd_raw,
    }

    if args.command in commands:
        commands[args.command](client, args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
