use crate::db::{
    attendance::Attendance,
    leaderboard::{CodeforcesStatsWithName, LeaderboardWithMember, LeetCodeStatsWithName},
    member::{Member, StreakUpdate},
};
use async_graphql::{Context, Object};
use chrono::NaiveDate;
use root::db::{
    attendance::{AttendanceStreak, AttendanceSummary, DailyCount, MemberAttendance},
    projects::ActiveProjects,
};
use sqlx::PgPool;
use std::sync::Arc;

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
            "SELECT id, date, timein, timeout, is_present 
            FROM Attendance WHERE date = $1",
        )
        .bind(date)
        .fetch_all(pool.as_ref())
        .await?;
        Ok(attendance_list)
    }
    async fn get_streak(&self, ctx: &Context<'_>, id: i32) -> Result<StreakUpdate, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let streak = sqlx::query_as::<_, StreakUpdate>("SELECT * FROM StreakUpdate WHERE id = $1")
            .bind(id)
            .fetch_one(pool.as_ref())
            .await?;

        Ok(streak)
    }

    async fn get_update_streak(&self, ctx: &Context<'_>) -> Result<Vec<StreakUpdate>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let streak = sqlx::query_as::<_, StreakUpdate>("SELECT * FROM StreakUpdate")
            .fetch_all(pool.as_ref())
            .await?;

        Ok(streak)
    }

    async fn get_attendance_streak(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceStreak>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let attendance_streak = sqlx::query_as::<_, AttendanceStreak>(
            "SELECT * from AttendanceStreak 
            WHERE month >= $1 AND month < $2
            ",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool.as_ref())
        .await?;

        Ok(attendance_streak)
    }

    async fn get_attendance_summary(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<AttendanceSummary, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");
        let attendance_days = sqlx::query_as::<_, (NaiveDate, i64)>(
            "SELECT date, COUNT(*) FROM Attendance 
            WHERE date >= $1 AND date < $2 
            AND is_present = true 
            GROUP BY date ORDER BY date",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool.as_ref())
        .await?;

        let member_attendance = sqlx::query_as::<_, (i32, i64)>(
            "SELECT id, COUNT(*) FROM Attendance 
            WHERE date >= $1 AND date < $2 
            AND is_present = true 
            GROUP BY id",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool.as_ref())
        .await?;

        let max_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM Attendance 
            WHERE date >= $1 AND date < $2 
            AND is_present = true",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool.as_ref())
        .await?;

        let daily_count = attendance_days
            .into_iter()
            .map(|(date, count)| DailyCount { date, count })
            .collect();

        let member_attendance = member_attendance
            .into_iter()
            .map(|(id, present_days)| MemberAttendance { id, present_days })
            .collect();

        let summaries: AttendanceSummary = AttendanceSummary {
            max_days: max_count[0],
            member_attendance,
            daily_count,
        };

        Ok(summaries)
    }

    pub async fn get_non_working_days(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<NaiveDate>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let dates = sqlx::query_scalar::<_, NaiveDate>(
            "SELECT date
            FROM Attendance
            GROUP BY date
            HAVING BOOL_AND(NOT is_present)
            ORDER BY date",
        )
        .fetch_all(pool.as_ref())
        .await?;

        Ok(dates)
    }

    pub async fn get_projects(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<ActiveProjects>, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let active_projects = sqlx::query_as::<_, ActiveProjects>("SELECT * FROM ActiveProjects")
            .fetch_all(pool.as_ref())
            .await?;

        Ok(active_projects)
    }
}
