# Shkolo Mobile App API Schema

**Base URL:** `https://api.shkolo.bg`
**App Version:** 1.43.3 (Build 460)
**Bundle Identifier:** `com.shkolo.mobileapp`

## Authentication

### Headers
All authenticated requests include:
```
Authorization: Bearer {token}
Accept: application/json
Content-Type: application/json
language: {bg|es}
School-Year: {year_id}  (optional)
user-agent: Shkolo-app-iOS/{version} | Shkolo-app-Android/{version}
```

Guest/unauthenticated requests may receive and should send:
```
X-Guest-Token: {guest_token}
```

---

## API Endpoints

### Authentication (`/v1/auth`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/auth/login` | Login with credentials |
| POST | `/v1/auth/logout` | Logout current session |
| POST | `/v1/auth/google` | OAuth login with Google |
| POST | `/v1/auth/facebook` | OAuth login with Facebook |
| POST | `/v1/auth/apple` | OAuth login with Apple |
| POST | `/v1/auth/microsoft` | OAuth login with Microsoft |
| POST | `/v1/auth/mon` | OAuth login with MON (Ministry of Education) |
| POST | `/v1/auth/passwordReset` | Request password reset |
| POST | `/v1/auth/fcmToken` | Register FCM push notification token |
| POST | `/v1/auth/authenticate-login` | Authenticate login request |
| POST | `/v1/auth/login-pending-token` | Login with pending token |
| POST | `/v1/auth/register-identity-login` | Register identity login |
| POST | `/v1/auth/switchUserAndYear` | Switch user and school year |
| GET | `/v1/auth/switchYear/{year_id}` | Switch school year |
| GET | `/v1/auth/users` | Get user list |
| GET | `/v1/auth/usersAndYears` | Get users and available years |
| GET | `/v1/auth/waiting-room-info` | Get waiting room info |
| GET | `/v1/auth/get-all-enabled-modules` | Get enabled modules for user |
| GET | `/v1/auth/get-sentry-options` | Get Sentry configuration |
| GET | `/v1/auth/download/{id}` | Download authenticated file |

#### Two-Factor Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/auth/twoFactorAuth/check?deviceUuid={uuid}` | Check 2FA status |
| GET | `/v1/auth/twoFactorAuth/get-awaiting-device` | Get awaiting device info |
| POST | `/v1/auth/twoFactorAuth/device` | Register 2FA device |
| POST | `/v1/auth/twoFactorAuth/send` | Send 2FA code |
| POST | `/v1/auth/twoFactorAuth/resend` | Resend 2FA code |
| POST | `/v1/auth/twoFactorAuth/verify` | Verify 2FA code |
| POST | `/v1/auth/twoFactorAuth/verifyMobile` | Verify mobile 2FA |

---

### Identity (`/v1/identity`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/identity` | Get current identity |
| GET | `/v1/identity/roles` | Get user roles |
| GET | `/v1/identity/schools` | Get available schools |
| GET | `/v1/identity/school/{school_id}` | Get specific school info |
| GET | `/v1/identity/school-class-levels/{school_id}` | Get class levels for school |
| GET | `/v1/identity/school-class-divisions/{school_id}` | Get class divisions for school |
| GET | `/v1/identity/class-levels` | Get class levels |
| GET | `/v1/identity/class-divisions` | Get class divisions |
| POST | `/v1/identity/add-new-role` | Add new role |
| POST | `/v1/identity/new-role` | Create new role |
| POST | `/v1/identity/match` | Match identity |
| PUT | `/v1/identity/password` | Update password |
| POST | `/v1/identity/send-sms-code` | Send SMS verification code |
| POST | `/v1/identity/verify-sms-code` | Verify SMS code |
| POST | `/v1/identity/resend-sms-code` | Resend SMS code |

---

### Profile (`/v1/profile`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/profile/all-roles` | Get all available roles |
| GET | `/v1/profile/all-pupils-under-fourteen` | Get pupils under 14 |
| GET | `/v1/profile/access-update-data` | Get access update data |
| GET | `/v1/profile/pending-images` | Get pending profile images |
| POST | `/v1/profile/edit` | Edit profile |
| POST | `/v1/profile/update` | Update profile |
| POST | `/v1/profile/email` | Update email |
| POST | `/v1/profile/email/code` | Verify email code |
| POST | `/v1/profile/phone` | Update phone |
| POST | `/v1/profile/phone/code` | Verify phone code |
| POST | `/v1/profile/resendSmsCode` | Resend SMS code |
| POST | `/v1/profile/access/update` | Update access permissions |
| POST | `/v1/profile/activate-pupil/{pupil_id}` | Activate pupil account |
| POST | `/v1/profile/approve-pupil/{pupil_id}` | Approve pupil account |
| POST | `/v1/profile/switch-language` | Switch app language |

---

### Diary (`/v1/diary`)

#### Class Years
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/classYears` | Get all class years |
| GET | `/v1/diary/classYears/{class_year_id}/pupils` | Get pupils in class year |
| GET | `/v1/diary/classYears/{class_year_id}/courses` | Get courses for class year |
| GET | `/v1/diary/classYears/{class_year_id}/mycourses` | Get my courses for class year |
| GET | `/v1/diary/classYears/{class_year_id}/grades` | Get grades for class year |
| GET | `/v1/diary/classYears/{class_year_id}/grades/summary/{term_id}` | Get grade summary |
| GET | `/v1/diary/classYears/{class_year_id}/grades/systems` | Get grading systems |
| GET | `/v1/diary/classYears/{class_year_id}/absences` | Get absences for class year |
| GET | `/v1/diary/classYears/{class_year_id}/absences?shi_ids={ids}&date={date}` | Get filtered absences |
| GET | `/v1/diary/classYears/{class_year_id}/absences/summary` | Get absences summary |
| GET | `/v1/diary/classYears/{class_year_id}/feedbacks` | Get feedbacks (badges/remarks) |
| GET | `/v1/diary/classYears/{class_year_id}/feedbacks/summary` | Get feedbacks summary |
| GET | `/v1/diary/classYears/{class_year_id}/scheduleHours?date={date}&groupByHour={bool}&myHours={bool}` | Get schedule hours |

#### Virtual Class Years
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/virtualClassYears/{id}/pupils` | Get pupils in virtual class |
| GET | `/v1/diary/virtualClassYears/{id}/courses` | Get courses |
| GET | `/v1/diary/virtualClassYears/{id}/mycourses` | Get my courses |
| GET | `/v1/diary/virtualClassYears/{id}/grades` | Get grades |
| GET | `/v1/diary/virtualClassYears/{id}/grades/summary/{term_id}` | Get grade summary |
| GET | `/v1/diary/virtualClassYears/{id}/grades/systems` | Get grading systems |
| GET | `/v1/diary/virtualClassYears/{id}/absences` | Get absences |
| GET | `/v1/diary/virtualClassYears/{id}/absences/summary` | Get absences summary |
| GET | `/v1/diary/virtualClassYears/{id}/feedbacks` | Get feedbacks |
| GET | `/v1/diary/virtualClassYears/{id}/feedbacks/summary` | Get feedbacks summary |
| GET | `/v1/diary/virtualClassYears/{id}/scheduleHours?date={date}&groupByHour={bool}&myHours={bool}` | Get schedule |

#### Pupils
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/pupils` | Get all pupils |
| GET | `/v1/diary/pupils/{pupil_id}/grades?mix_id={mix_id}` | Get pupil grades |
| GET | `/v1/diary/pupils/{pupil_id}/grades/summary` | Get pupil grade summary |
| GET | `/v1/diary/pupils/{pupil_id}/absences?mix_id={mix_id}&virtual_class_year_id={id}` | Get pupil absences |
| GET | `/v1/diary/pupils/{pupil_id}/absences/summary` | Get pupil absences summary |
| GET | `/v1/diary/pupils/{pupil_id}/feedbacks?mix_id={mix_id}&shi_id={shi_id}&virtual_class_year_id={id}` | Get pupil feedbacks |
| GET | `/v1/diary/pupils/{pupil_id}/feedbacks/summary` | Get pupil feedbacks summary |
| GET | `/v1/diary/pupils/{pupil_id}/scheduleHours?date={date}` | Get pupil schedule |

#### Schedule Hours
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/scheduleHours?date={date}` | Get schedule for date |
| GET | `/v1/diary/scheduleHours/{shi_id}/details` | Get hour details |
| GET | `/v1/diary/scheduleHours/{shi_id}/topics` | Get topics for hour |
| GET | `/v1/diary/scheduleHours/{shi_id}/absences` | Get absences for hour |
| GET | `/v1/diary/scheduleHours/{shi_id}/feedbacks` | Get feedbacks for hour |
| GET | `/v1/diary/scheduleHours/{shi_id}/grades` | Get grades for hour |
| POST | `/v1/diary/scheduleHours/{shi_id}/attendance` | Record attendance |
| POST | `/v1/diary/scheduleHours/{shi_id}/teacherHourAttendance` | Record teacher attendance |
| POST | `/v1/diary/scheduleHours/{shi_id}/teacherHourDescription` | Add hour description |
| POST | `/v1/diary/scheduleHours/{shi_id}/type` | Set hour type |
| POST | `/v1/diary/scheduleHours/{shi_id}/topics` | Add topics |
| POST | `/v1/diary/scheduleHours/{shi_id}/absences` | Add absences |
| POST | `/v1/diary/scheduleHours/{shi_id}/feedbacks` | Add feedbacks |
| POST | `/v1/diary/scheduleHours/{shi_id}/grades` | Add grades |
| POST | `/v1/diary/scheduleHours/{shi_id}/homework` | Add homework |
| GET | `/v1/diary/scheduleHours/homework/{homework_id}/details` | Get homework details |

#### Grades
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/grades/types` | Get grade types |
| POST | `/v1/diary/grades/{grade_id}` | Update grade |
| DELETE | `/v1/diary/grades/{grade_id}` | Delete grade |

#### Other Diary Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/feedbacks/badges` | Get available badges |
| GET | `/v1/diary/notTakenHours` | Get not taken hours |
| GET | `/v1/diary/medicalNotes/{id}` | Get medical notes |
| GET | `/v1/diary/homeworks/courses` | Get homework courses |
| GET | `/v1/diary/homeworks/list/{class_year_id}` | Get homework list |

#### Correction Exams
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/diary/correction-exam/get/{id}` | Get correction exam |
| GET | `/v1/diary/correction-exam/getById/{id}` | Get correction exam by ID |
| GET | `/v1/diary/correction-exam/manage/get/{id}` | Get managed correction exam |
| POST | `/v1/diary/correction-exam/insertGrade` | Insert correction exam grade |
| POST | `/v1/diary/correction-exam/managePost/{id}` | Manage correction exam |
| POST | `/v1/diary/correction-exam/flush/{id}` | Flush correction exam |

---

### Events (`/v1/events`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/events` | Get all events |
| GET | `/v1/events?is_school_calendar=1` | Get school calendar events |
| GET | `/v1/events/{event_id}` | Get specific event |
| GET | `/v1/events/{event_id}/invitations?offset={offset}&limit={limit}` | Get event invitations |
| GET | `/v1/events/archivedEvents` | Get archived events |
| GET | `/v1/events/invitations` | Get user's invitations |
| GET | `/v1/events/invitations?pupil_user_id={id}` | Get pupil's invitations |

---

### Messenger (`/v1/messenger`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/messenger/folders` | Get message folders |
| GET | `/v1/messenger/threads?folderId={folder_id}` | Get threads in folder |
| GET | `/v1/messenger/threads/{thread_id}/messages` | Get thread with messages |
| GET | `/v1/messenger/recipients` | Get available recipients |
| GET | `/v1/messenger/canSendMessages` | Check if user can send messages |
| POST | `/v1/messenger/threads` | Create new thread/message |
| POST | `/v1/messenger/threads/{thread_id}` | Reply to thread |
| POST | `/v1/messenger/threads/archiveThreads` | Archive threads |
| POST | `/v1/messenger/threads/leaveThreads` | Leave threads |
| POST | `/v1/messenger/threads/markReadOrUnread` | Mark threads read/unread |
| POST | `/v1/messenger/threads/moveThreadsToFolder` | Move threads to folder |
| POST | `/v1/messages/refactoring/postThreadLastRead` | Update last read position |

---

### Notifications (`/v1/notifications`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/notifications?page={page}` | Get notifications (paginated) |
| GET | `/v1/notifications/unread-count` | Get unread count |
| POST | `/v1/notifications/markAsRead` | Mark notification as read |
| POST | `/v1/notifications/markAllAsRead` | Mark all as read |

---

### Tasks (`/v1/tasks`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/tasks` | Get all tasks |
| GET | `/v1/tasks/{task_id}` | Get specific task |
| GET | `/v1/tasks/my-tasks` | Get user's tasks |
| GET | `/v1/tasks/assigned` | Get assigned tasks |
| GET | `/v1/tasks/assigned/{id}` | Get specific assigned task |
| GET | `/v1/tasks/instance/{instance_id}` | Get task instance |
| GET | `/v1/tasks/participants-list?invitationType={type}` | Get participants list |
| POST | `/v1/tasks/create` | Create new task |
| POST | `/v1/tasks/change-status` | Change task status |
| POST | `/v1/tasks/batch-change-status` | Batch change task status |
| POST | `/v1/tasks/instance/post-comment` | Post comment on task |

---

### Tests (`/v1/tests`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/tests` | Get all tests |
| GET | `/v1/tests/{test_id}` | Get specific test |
| GET | `/v1/tests/my-tests?testCreatorFilter=1` | Get my created tests |
| GET | `/v1/tests/filters` | Get test filters |
| GET | `/v1/tests/results/my-results` | Get my test results |
| GET | `/v1/tests/results/child/{child_id}` | Get child's test results |
| POST | `/v1/tests/execution/question` | Submit test answer |
| POST | `/v1/tests/execution/finish` | Finish test execution |

---

### Activities (`/v1/activity`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/activity/{activity_id}` | Get activity details |
| GET | `/v1/activity/all-activities?search={query}` | Search all activities |
| GET | `/v1/activity/my-activities?search={query}` | Search my activities |
| GET | `/v1/activity/pinned-activities` | Get pinned activities |
| GET | `/v1/activity/pupil-activities/{pupil_id}` | Get pupil's activities |
| GET | `/v1/activity/instance/{instance_id}` | Get activity instance |

---

### Teacher Absences (`/v1/teacher-absence`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/teacher-absence/details/{id}` | Get teacher absence details |
| GET | `/v1/teacher-absence/my-replacements?page={page}` | Get my replacements |

---

### Virtual School (`/v1/virtualschool`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/virtualschool/virtualrooms` | Get virtual rooms |
| GET | `/v1/virtualschool/virtualrooms/{room_id}` | Get virtual room details |

---

### Files (`/v1/files`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/files` | Get files |
| GET | `/v1/files/{file_id}` | Get specific file |
| GET | `/v1/files/download/thread/{thread_id}` | Download thread files |
| POST | `/v1/files` | Upload file |

---

### Users (`/v1/users`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/users/{user_id}` | Get user details |

---

### Statistics (`/v1/stats`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/stats` | Get statistics |

---

### Cities (`/v1/cities`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/cities/getCitiesByTypedText?typedText={query}` | Search cities |

---

## Data Types & Constants

### Role IDs
| ID | Role |
|----|------|
| 1 | System Admin |
| 2 | Director |
| 3 | Deputy Director |
| 4 | Accountant |
| 5 | Secretary |
| 6 | Treasurer |
| 7 | Stoker |
| 8 | Health Officer |
| 9 | Host |
| 10 | Librarian |
| 11 | Teacher |
| 12 | Parent |
| 13 | Student/Pupil |
| 14 | Other |
| 15 | Laborer |
| 16 | Resource Teacher |
| 17 | ZATS |
| 18 | Teacher CDO |
| 19 | Medical GP |
| 20 | Counselor |
| 21 | Speech Therapist |
| 22 | Psychologist |

### Event Types
| ID | Type |
|----|------|
| 1 | Parent Meeting |
| 5 | School Event |
| 7 | Administrative |
| 8 | State Matriculation Exam |
| 9 | National External Exam |
| 10 | Official Holiday |
| 11 | Holiday |
| 12 | Homework |
| 13 | Control Test |
| 14 | Class Test |
| 15 | Competition |
| 16 | Qualification Exam |

### Grade Types
| ID | Type |
|----|------|
| 5 | Term Grade |
| 6 | Year/Annual Grade |

### Grade Modes
| ID | Mode |
|----|------|
| 1 | SEN (Special Educational Needs) |
| 2 | Junior |
| 3 | Improvement Excellent |
| 4 | 1-5 Scale |
| 5 | Regular BG |
| 6 | Regular ES |

### Absence Types
| ID | Type |
|----|------|
| 1 | Excused |
| 2 | Delay |
| 3 | Unexcused |

### Excused Absence Reasons
| ID | Reason |
|----|--------|
| 1 | Health |
| 2 | Family |
| 3 | Other |

### Task Status
| ID | Status |
|----|--------|
| 1 | Assigned |
| 2 | In Progress |
| 3 | Completed |
| 4 | Incompleted |

### Identity Status
| ID | Status |
|----|--------|
| 1 | Waiting Activation |
| 2 | Inactive Pending Input |
| 3 | Waiting Approval |
| 4 | Active Without Email Confirmation |
| 5 | Active |

### Test Question Types
| ID | Type |
|----|------|
| 1 | Single Choice |
| 2 | Multiple Choice |
| 3 | Number Input |
| 4 | Short Answer |
| 5 | Long Answer |
| 6 | Drag and Drop |

### Test Status
| ID | Status |
|----|--------|
| 1 | Ongoing |
| 2 | Answered |
| 3 | Assessed Auto |
| 4 | Assessed Manual |

### Modules
- `modules.diary`
- `modules.communication`
- `modules.topic_plans`
- `modules.lecturer_hours`
- `modules.statistics`
- `modules.pupil_documents`
- `modules.digital_content`
- `modules.activities`
- `modules.events`
- `modules.tests`
- `modules.inventory`
- `modules.payments`
- `modules.competitions`
- `modules.virtual_school`
- `modules.tasks`
- `modules.print_blanks`
- `modules.compulsory_education_book`
- `modules.principal_order_book`
- `modules.teacher_portfolio`
- `modules.main_book`
- `modules.correspondence_book`

---

## OAuth Configuration

### Google
- Client ID: `186341692533-14k2gd4i6fsj230cqu40jf04dp0igr3j.apps.googleusercontent.com`

### Microsoft
- Client ID: `7025629c-3a86-4d58-8894-06ea9208ea1c`

### MON (Ministry of Education Bulgaria)
- Client ID: `b6c7b0f4-5554-445c-b670-4e3e66cba6af`
- Tenant ID: `420584ab-4eec-41c7-bb43-7364d0a6fdfd`

### Facebook
- App ID: `1301157866630854`

---

## Date Formats

- API Date: `YYYY-MM-DD`
- API DateTime: `DD.MM.YYYY HH:mm:ss`
- BG Locale: `DD.MM.YYYY` / `DD.MM` (short)
- ES Locale: `DD/MM/YYYY` / `DD/MM` (short)

---

## External Services

- **Firebase Messaging** - Push notifications
- **Sentry** - Error tracking
- **CodePush** - OTA updates (Deployment Key: `XLZWD4qY7FvoOCS5ItZpBgeYoPGKeJzU5qhCq`)
- **JitsiMeet SDK** - Video conferencing for virtual school
- **Giphy SDK** - GIF support in messaging

---

*Schema extracted from Shkolo iOS app v1.43.3 (Build 460)*
*Generated: 2026-02-16*
