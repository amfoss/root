use async_graphql::SimpleObject;
use chrono::{NaiveDate, NaiveTime};
use sqlx::FromRow;

//Struct for the Attendance table
#[derive(FromRow, SimpleObject)]
pub struct Attendance {
    pub id: i32,
    pub date: NaiveDate,
    pub timein: NaiveTime,
    pub timeout: NaiveTime,
    pub is_present: bool,
}

#[derive(FromRow, SimpleObject)]
pub struct AttendanceStreak {
    pub id: i32,
    pub member_id: i32,
    pub month: NaiveDate,
    pub streak: i32,
}

#[derive(FromRow, SimpleObject)]
pub struct AttendanceSummary {
    pub max_days: i64,
    pub member_attendance: Vec<MemberAttendance>,
    pub daily_count: Vec<DailyCount>,
}

#[derive(FromRow, SimpleObject)]
pub struct MemberAttendance {
    pub id: i32,
    pub present_days: i64,
}

#[derive(FromRow, SimpleObject)]
pub struct DailyCount {
    pub date: NaiveDate,
    pub count: i64,
}
