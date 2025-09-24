use async_graphql::{Context, Object};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error};
use crate::db::leaderboard::{CodeforcesStats, LeetCodeStats};

pub struct LeadMutation;

#[Object]
impl LeadMutation {
    pub async fn add_or_update_leetcode_username(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        username: String,
    ) -> Result<LeetCodeStats, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        sqlx::query_as::<_, LeetCodeStats>(
            "
            INSERT INTO leetcode_stats (member_id, leetcode_username, problems_solved, easy_solved, medium_solved, hard_solved, contests_participated, best_rank, total_contests)
            VALUES ($1, $2, 0, 0, 0, 0, 0, 0, 0)
            ON CONFLICT (member_id) DO UPDATE
            SET leetcode_username = EXCLUDED.leetcode_username
            RETURNING *
            ",
        )
        .bind(member_id)
        .bind(username)
        .fetch_one(pool.as_ref())
        .await
    }

    async fn add_or_update_codeforces_handle(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        handle: String,
    ) -> Result<CodeforcesStats, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        sqlx::query_as::<_, CodeforcesStats>(
            "
            INSERT INTO codeforces_stats (member_id, codeforces_handle, codeforces_rating, max_rating, contests_participated)
            VALUES ($1, $2, 0, 0, 0)
            ON CONFLICT (member_id) DO UPDATE
            SET codeforces_handle = EXCLUDED.codeforces_handle
            RETURNING *  
            ",
        )
        .bind(member_id)
        .bind(handle)
        .fetch_one(pool.as_ref())
        .await
    }
}
