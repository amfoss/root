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
# Get attendance by member ID during a certain time period
query {
    member (memberId: 1) {
        attendance (startDate: ,endDate: ) {
            records {
                attendanceId
                date
                isPresent
                timeIn
                timeOut
            }
        }
    }
}
```

Get all attendance for a specific date

```graphql
query {
  allMembers {
    memberId
    name
    attendance {
      records(startDate: , endDate: ) {
        attendanceId
        date
        isPresent
        timeIn
        timeOut
      }
    }
  }
}
```

Get absent and present count for member during a time period

```graphql
query {
  member (memberId: ) {
    memberId
    name
    attendance {
        records {
            presentCount(startDate: ,endDate: )
            absentCount(startDate: ,endDate: )
        }
    }
  }
}
```

Get all members attendance for a particular date
```graphql
query {
    allMembers {
        memberId
        name
        attendance {
            onDate (date: ) {
                isPresent
                timeIn
                timeOut
            }
        }
    }
}
```

Get present or absent count of a particular time period
```graphql
query {
    allMembers {
        attendance {
            presentCount(startDate: ,endDate: )
            absentCount(startDate: ,endDate: )
        }
    }
}
```

### Mark Attendance
Record a member's attendance for the day.

```graphql
mutation {
    markAttendance(
            memberId: 1
            date: "2025-01-15"
            timeIn: "09:00:00"
            timeOut: "17:00:00"      
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
