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

#[derive(InputObject)]
pub struct MarkLeaveInput {
    pub discord_id: String,
    pub reason: String,
    pub duration: i32,
    pub approved_by: Option<String>,
}

#[derive(SimpleObject, FromRow)]
pub struct MarkLeaveOutput {
    pub discord_id: String,
    pub date: NaiveDate,
    pub reason: String,
    pub duration: i32,
    pub approved_by: Option<String>,
}