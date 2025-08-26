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

## Daily Task

The `src/daily_task/daily_task.rs` system automatically updates attendance summaries at midnight.
