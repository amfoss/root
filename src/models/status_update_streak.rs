use async_graphql::{InputObject, SimpleObject};
use sqlx::FromRow;

#[derive(SimpleObject, FromRow)]
pub struct StatusUpdateStreak {
    pub member_id: i32,
    pub current_streak: i32,
    pub max_streak: i32,
}

#[derive(SimpleObject, FromRow)]
pub struct StatusUpdateStreakInfo {
    pub current_streak: i32,
    pub max_streak: i32,
}

#[derive(InputObject)]
pub struct IncrementStreakInput {
    pub member_id: Option<i32>,
    pub discord_id: Option<String>,
}
