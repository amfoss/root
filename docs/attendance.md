# Attendance System

Track daily member attendance and generate monthly attendance summaries. The summaries are used for quick access to see how many days a member has attended in a specific month, used in the amD attendance report.

## Models

### Attendance
```rust
struct Attendance {
    attendance_id: i32,
    member_id: i32,
    date: NaiveDate,
    is_present: bool,
    time_in: Option<NaiveTime>,
    time_out: Option<NaiveTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}
```
The final two fields are not exposed in the interface for obvious reasons.

### AttendanceSummary
Monthly attendance summary for each member.
```rust
struct AttendanceSummary {
    member_id: i32,
    year: i32,
    month: i32,
    days_attended: i32,
}
```

### Daily Count
Total Lab count for each date
```rust
pub struct DailyCount {
    pub date: String,
    pub count: i64,
}
```

### Member Summary
Total lab members attended in the past 6 months
```rust
pub struct MemberAttendanceSummary {
    pub id: i32,
    pub name: String,
    pub present_days: i64,
}
```

### Attendance Report
Attendance report of the club members
```rust
pub struct AttendanceReport {
    pub daily_count: Vec<DailyCount>,
    pub member_attendance: Vec<MemberAttendanceSummary>,
    pub max_days: i64,
}
```

## Queries

### Get Attendance
Retrieve attendance records by member ID or date.

```graphql
# Get attendance by member ID
query {
    attendance(memberId: 1) {
        attendanceId
        date
        isPresent
        timeIn
        timeOut
    }
}
```

Get all attendance for a specific date

```graphql
query {
    attendanceByDate(date: "2025-02-27") {
        attendanceId
        memberId
        name
        year
        isPresent
        timeIn
        timeOut
    }
}
```

### Mark Attendance
Record a member's attendance for the day.

```graphql
mutation {
    markAttendance(
        input: {
            memberId: 1
            date: "2025-01-15"
            timeIn: "09:00:00"
            timeOut: "17:00:00"
        }
    ) {
        attendanceId
        isPresent
        timeIn
        timeOut
    }
}
```

### Get Attendance Summary
Get monthly attendance summary for a member.

```graphql
query {
    attendanceSummary(memberId: 1) {
        year
        month
        daysAttended
    }
}
```

### Get Attendance Report
Get Attendance report containing lab count and members attendance report of the past 6 months.
`maxDays returns the count of days when lab was open in the past 6 months`
```graphql
query{
  getAttendanceSummary(startDate:"2024-12-20", endDate: "2024-12-27"){
    memberAttendance{
      name,
      presentDays
    }
    dailyCount{
      date,
      count
    }
    maxDays
  }
}
```


## Daily Task

The `src/daily_task/daily_task.rs` system automatically updates attendance summaries at midnight.
