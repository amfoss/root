# Member Management

Manage club member profiles. This is the central entity for the database. Most things should relate to one or more members.

## Models

### Member
```rust
struct Member {
    member_id: i32,
    roll_no: String,
    name: String,
    email: String,
    sex: Sex,
    year: i32,
    hostel: String,
    mac_address: String,
    discord_id: String,
    group_id: i32,
    track: String,
}
```

## Queries

All queries are split into allMembers and member root queries
allMembers can be filtered by year or track, while member needs memberId or email

### Details of all members
Retrieve the details of all members optionally fitered by year or track
```graphql
query {
    allMembers (year, track) {
        memberId
        rollIo
        name
        email
        sex
        year
        hostel
        macAddress
        discordId
        groupId
        track
    }
}
```

### Details of single member
Retrieve a single member's details using their either email or memberId
```graphql
query {
    member (memberId:00, email:"something@gmail.com" ) {
        memberId
        rollNo
        name
        email
        sex
        year
        hostel
        macAddress
        discordId
        groupId
        track
    }
}
```

## Mutations

### Create Member
Add a new member to the database.
```graphql
mutation {
    createMember(
        input: {
            rollNo: "AM.XX.U4XXX"
            name: "John Doe"
            email: "john@amfoss.in"
            sex: "M"
            year: 2
            hostel: "MH"
            macAddress: "XX:XX:XX:XX:XX:XX"
            discordId: "123456789"
            groupId: 1
            track: "web"
        }
    ) {
        memberId
        name
    }
}
``` 

### Update Member
Update details of an existing member
```graphql
mutation {
    updateMember (
        input: {
            memberId
            rollNo
            name
            email
            sex
            year
            hostel
            macAddress
            discordId
            groupId
            track
        }
    ) {
        memberId
        name
    }
}
```