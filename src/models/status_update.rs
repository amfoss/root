use async_graphql::{InputObject, SimpleObject};
use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(SimpleObject, FromRow)]
#[graphql(complex)]
pub struct StatusUpdateRecord {
    pub update_id: i32,
    pub member_id: i32,
    pub date: NaiveDate,
    pub is_sent: bool,
}

#[derive(SimpleObject, FromRow)]
pub struct StatusUpdateStreakRecord {
    pub current_streak: Option<i64>,
    pub max_streak: Option<i64>,
}

#[derive(SimpleObject, FromRow)]
pub struct StatusBreakRecord {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub year: Option<i32>,
    pub member_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(InputObject)]
pub struct CreateStatusBreakInput {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub year: Option<i32>,
    pub member_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(SimpleObject, FromRow, Debug, Clone, PartialEq, Eq)]
pub struct MemberLifeStatusRecord {
    pub member_id: i32,
    pub lives: i32,
    pub recovery_streak: i32,
    pub is_probation: bool,
    pub last_reset_month: i32,
}
