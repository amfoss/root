use async_graphql::{Context, Object};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::sync::Arc;

use crate::db::{
    attendance::Attendance, member::StreakUpdate,
    leaderboard::{CodeforcesStatsWithName, LeaderboardWithMember, LeetCodeStatsWithName},
    member::Member,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    //Query for retrieving the members
    async fn get_member(&self, ctx: &Context<'_>) -> Result<Vec<Member>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let users = sqlx::query_as::<_, Member>("SELECT * FROM Member")
            .fetch_all(pool.as_ref())
            .await?;
        Ok(users)
    }

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
            JOIN member m ON l.member_id = m.id
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
            JOIN member m ON l.member_id = m.id
            ORDER BY  best_rank
            ",
        )
        .fetch_all(pool.as_ref())
        .await?;
        Ok(leetcode_stats)
    }

    async fn get_codeforces_stats(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<CodeforcesStatsWithName>, sqlx::Error> {
        // let pool = ctx.data::<PgPool>()?;
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let codeforces_stats = sqlx::query_as::<_, CodeforcesStatsWithName>(
            "SELECT c.*, m.name AS member_name
            FROM codeforces_stats c
            JOIN member m ON c.member_id = m.id
            ORDER BY max_rating DESC",
        )
        .fetch_all(pool.as_ref())
        .await?;
        Ok(codeforces_stats)
    }

    //Query for retrieving the attendance based on date
    async fn get_attendance(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> Result<Vec<Attendance>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let attendance_list = sqlx::query_as::<_, Attendance>(
            "SELECT id, date, timein, timeout, is_present FROM Attendance WHERE date = $1",
        )
        .bind(date)
        .fetch_all(pool.as_ref())
        .await?;
        Ok(attendance_list)
    }
    async fn get_streak(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<StreakUpdate, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");
        let streak = sqlx::query_as::<_, StreakUpdate>("SELECT * FROM StreakUpdate WHERE id = $1")
        .bind(id)       
        .fetch_one(pool.as_ref())
        .await?;

        Ok(streak)
    }
}
