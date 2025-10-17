# Status Update Streaks

## Overview
Track members' daily status update streaks and records.

## Models

### StatusUpdateStreak
```rust
struct StatusUpdateStreak {
    member_id: i32,
    current_streak: i32,
    max_streak: i32,
}
```

## Queries

### Members records in a time period
Retrieve members status update records in a given time period
```graphql
query {
  member (memberId: ) {
    memberId
    name
    status {
      records(startDate: ,endDate: ) {
        isSent
        updateId
        memberId
        date
      }
    }
  }
}
```

###  Members records on a date
Retrieve members status update records on a certain date
```graphql
query {
  allMembers {
    memberId
    name
    status {
      onDate (date: ) {
        isSent
        updateId
        memberId
        date
      }
    }
  }
}
```

## Mutations

### Increment Streak
```graphql
mutation {
    incrementStreak(emails: ) {
        memberId
        date
        isSent
    }
}
```