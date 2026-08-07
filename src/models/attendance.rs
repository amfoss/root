use async_graphql::{InputObject, SimpleObject};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::FromRow;

#[derive(SimpleObject, FromRow)]
pub struct AttendanceRecord {
    pub attendance_id: i32,
    pub date: NaiveDate,
    pub is_present: bool,
    pub time_in: Option<NaiveTime>,
    pub time_out: Option<NaiveTime>,
    #[graphql(skip)] // Don't expose internal fields/meta-data
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    pub updated_at: NaiveDateTime,
}

#[derive(InputObject)]
pub struct MarkAttendanceInput {
    pub member_id: i32,
    pub date: NaiveDate,
    pub hmac_signature: String,
}

#[derive(SimpleObject, FromRow)]
pub struct LeaveRecord {
    pub discord_id: String,
    pub from_date: NaiveDate,
    pub applied_at: NaiveDateTime,
    pub reason: String,
    pub duration: i32,
    pub approved_by: Option<String>,
}

#[derive(SimpleObject)]
pub struct CheckLeave {
    pub message_id: i64,
    pub approved_by: String,
}