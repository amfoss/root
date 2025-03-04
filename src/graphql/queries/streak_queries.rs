use std::sync::Arc;

use crate::models::status_update_streak::StatusUpdateStreak as Streak;
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

#[derive(Default)]
pub struct StreakQueries;

#[Object]
impl StreakQueries {
    async fn streak(&self, ctx: &Context<'_>, member_id: i32) -> Result<Streak> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        Ok(sqlx::query_as::<_, Streak>(
            "SELECT current_streak, max_streak FROM StatusUpdateStreak WHERE member_id = $1",
        )
        .bind(member_id)
        .fetch_one(pool.as_ref())
        .await?)
    }

    }
}
