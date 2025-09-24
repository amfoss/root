use async_graphql::{Context, Object};
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::leaderboard::{
    CodeforcesStatsWithName, LeaderboardWithMember, LeetCodeStatsWithName,
};

#[derive(Default)]
pub struct LeaderboardQueries;

#[Object]
impl LeaderboardQueries {
    async fn get_unified_leaderboard(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<LeaderboardWithMember>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let leaderboard = sqlx::query_as::<_, LeaderboardWithMember>(
            "SELECT l.*, m.name AS member_name
            FROM leaderboard l
            JOIN member m ON l.member_id = m.member_id
           ORDER BY unified_score DESC",
        )
        .fetch_all(pool.as_ref())
        .await?;
        Ok(leaderboard)
    }

    async fn get_leetcode_stats(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<LeetCodeStatsWithName>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let leetcode_stats = sqlx::query_as::<_, LeetCodeStatsWithName>(
            "SELECT l.*, m.name AS member_name
            FROM leetcode_stats l
            JOIN member m ON l.member_id = m.member_id
            ORDER BY best_rank",
        )
        .fetch_all(pool.as_ref())
        .await?;
        Ok(leetcode_stats)
    }

    async fn get_codeforces_stats(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<CodeforcesStatsWithName>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let codeforces_stats = sqlx::query_as::<_, CodeforcesStatsWithName>(
            "SELECT c.*, m.name AS member_name
            FROM codeforces_stats c
            JOIN member m ON c.member_id = m.member_id
            ORDER BY max_rating DESC",
        )
        .fetch_all(pool.as_ref())
        .await?;
        Ok(codeforces_stats)
    }
}
