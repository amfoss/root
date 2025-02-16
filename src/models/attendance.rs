use async_graphql::{InputObject, SimpleObject};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::FromRow;

#[derive(SimpleObject, FromRow)]
pub struct Attendance {
    pub attendance_id: i32,
    pub member_id: i32,
    pub date: NaiveDate,
    pub is_present: bool,
    pub time_in: Option<NaiveTime>,
    pub time_out: Option<NaiveTime>,
    #[graphql(skip)] // Don't expose internal fields/meta-data
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    pub updated_at: NaiveDateTime,
}

#[derive(SimpleObject, FromRow)]
pub struct AttendanceSummary {
    pub member_id: i32,
    pub year: i32,
    pub month: i32,
    pub days_attended: i32,
}

/// This struct is used in place of [`Attendance`] in nested queries to avoid overfetching.
#[derive(SimpleObject, FromRow)]
pub struct AttendanceInfo {
    pub date: NaiveDate,
    pub is_present: bool,
    pub time_in: Option<NaiveTime>,
    pub time_out: Option<NaiveTime>,
}

/// This struct is used in place of [`AttendanceSummary`] in nested queries to avoid overfetching.
#[derive(SimpleObject, FromRow)]
pub struct AttendanceSummaryInfo {
    pub year: i32,
    pub month: i32,
    pub days_attended: i32,
}

/// This struct is used to deserialize the input recieved for mutations on attendance.
#[derive(InputObject)]
pub struct MarkAttendanceInput {
    pub member_id: i32,
    pub date: NaiveDate,
    pub hmac_signature: String,
}

#[derive(SimpleObject)]
pub struct DailyCount {
    pub date: String,
    pub count: i64,
}

#[derive(SimpleObject)]
pub struct MemberAttendanceSummary {
    pub id: i32,
    pub name: String,
    pub present_days: i64,
}

#[derive(SimpleObject)]
pub struct AttendanceReport {
    pub daily_count: Vec<DailyCount>,
    pub member_attendance: Vec<MemberAttendanceSummary>,
    pub max_days: i64,
}

/// This struct combines attendance data with member name for queries that need both.
/// It joins the Attendance table with Member to include the member's name.
#[derive(SimpleObject, FromRow)]
pub struct AttendanceWithMember {
    pub attendance_id: i32,
    pub member_id: i32,
    pub date: NaiveDate,
    pub is_present: bool,
    pub time_in: Option<NaiveTime>,
    pub time_out: Option<NaiveTime>,
    pub name: String,
    pub year: i32,
}
