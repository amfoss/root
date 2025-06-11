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

### Get Member
Retrieve member details by ID, roll number, or Discord ID.

```graphql
query {
    getMember(rollNo: "AM.XX.U4XXX") {
        name
        email
        year
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
