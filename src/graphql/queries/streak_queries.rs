use std::sync::Arc;

use crate::models::{member::Member, status_update_streak::StatusUpdateStreak as Streak};
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

#[derive(Default)]
pub struct StreakQueries;

#[Object]
impl StreakQueries {
    async fn streak(
        &self,
        ctx: &Context<'_>,
        member_id: Option<i32>,
        roll_no: Option<String>,
        discord_id: Option<String>,
    ) -> Result<Streak> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        if let Some(id) = member_id {
            let streak_query = sqlx::query_as::<_, Streak>(
                "SELECT current_streak, max_streak FROM StatusUpdateStreak WHERE member_id = $1",
            )
            .bind(id)
            .fetch_one(pool.as_ref())
            .await?;

            return Ok(streak_query);
        }

        let member_query = if let Some(roll) = roll_no {
            sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE roll_no = $1")
                .bind(roll)
                .fetch_one(pool.as_ref())
                .await
        } else if let Some(discord) = discord_id {
            sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE discord_id = $1")
                .bind(discord)
                .fetch_one(pool.as_ref())
                .await
        } else {
            return Err(async_graphql::Error::new(
                "At least one key (member_id, roll_no, discord_id) must be specified.",
            ));
        };

        let member = match member_query {
            Ok(member) => member,
            Err(_) => {
                return Err(async_graphql::Error::new(
                    "No member found with the given criteria.",
                ))
            }
        };

        let streak_query = sqlx::query_as::<_, Streak>(
            "SELECT current_streak, max_streak FROM Streak WHERE member_id = $1",
        )
        .bind(member.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(streak_query)
    }
}
