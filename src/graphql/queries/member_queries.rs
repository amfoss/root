use crate::models::attendance::AttendanceRecord;
use async_graphql::{ComplexObject, Context, Object, Result};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::{member::Member, status_update::StatusUpdateStreakRecord};

#[derive(Default)]
pub struct MemberQueries;

pub struct StatusInfo {
    member_id: i32,
}

pub struct AttendanceInfo {
    member_id: i32,
}

#[Object]
impl MemberQueries {
    pub async fn all_members(
        &self,
        ctx: &Context<'_>,
        year: Option<i32>,
        track: Option<String>,
    ) -> Result<Vec<Member>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let mut query = sqlx::QueryBuilder::new("SELECT * FROM Member WHERE 1=1");

        if let Some(y) = year {
            query.push(" AND year = ");
            query.push_bind(y);
        }

        if let Some(g) = track {
            query.push(" AND track = ");
            query.push_bind(g);
        }

        let members = query
            .build_query_as::<Member>()
            .fetch_all(pool.as_ref())
            .await?;

        Ok(members)
    }

    async fn member(
        &self,
        ctx: &Context<'_>,
        member_id: Option<i32>,
        email: Option<String>,
    ) -> Result<Option<Member>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        match (member_id, email) {
            (Some(id), None) => {
                let member =
                    sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE member_id = $1")
                        .bind(id)
                        .fetch_optional(pool.as_ref())
                        .await?;
                Ok(member)
            }
            (None, Some(email)) => {
                let member = sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE email = $1")
                    .bind(email)
                    .fetch_optional(pool.as_ref())
                    .await?;
                Ok(member)
            }
            (Some(_), Some(_)) => Err("Provide only one of member_id or email".into()),
            (None, None) => Err("Provide either member_id or email".into()),
        }
    }
}

#[Object]
impl StatusInfo {
    async fn streak(&self, ctx: &Context<'_>) -> Result<StatusUpdateStreakRecord> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        // The below is based on the classic 'islands and gaps' problem, adapted to fit our needs.
        // The key idea used here is in the 'streaks' CTE: for consecutive dates (a streak), the difference
        // between the date value and its row number (rn) remains constant.
        // All rows with the same (date - rn) value therefore belong to the same streak.
        let result = sqlx::query_as::<_, StatusUpdateStreakRecord>(
            "WITH numbered AS (
                SELECT
                    date,
                    ROW_NUMBER() OVER (ORDER BY date) AS rn
                FROM statusupdatehistory
                WHERE member_id = $1
                  AND is_updated = true
            ),
            streaks AS (
                SELECT
                    date,
                    date - rn * INTERVAL '1 day' AS streak_id
                FROM numbered
            ),
            grouped AS (
                SELECT
                    COUNT(*) AS streak,
                    MAX(date) AS end_date
                FROM streaks
                GROUP BY streak_id
            )
            SELECT
                MAX(streak) AS max_streak,
                (
                    SELECT streak
                    FROM grouped
                    WHERE end_date = (
                        SELECT MAX(end_date)
                        FROM grouped
                    )
                ) AS current_streak
            FROM grouped;
        ",
        )
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(result)
    }

    async fn update_count(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let result : i64 = sqlx::query_scalar("SELECT count(*) AS updatecount FROM statusupdatehistory WHERE is_updated = TRUE and member_id=$1 and date BETWEEN $2 and $3;")
            .bind(self.member_id)
            .bind(start_date)
            .bind(end_date)
            .fetch_one(pool.as_ref())
            .await?;

        Ok(result)
    }
}

#[Object]
impl AttendanceInfo {
    async fn records(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceRecord>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let rows = sqlx::query_as::<_, AttendanceRecord>("SELECT * FROM Attendance att INNER JOIN member m ON att.member_id = m.member_id where date BETWEEN $1 and $2 AND att.member_id=$3")
        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_all(pool.as_ref())
        .await?;

        Ok(rows)
    }

    async fn on_date(&self, ctx: &Context<'_>, date: NaiveDate) -> Result<AttendanceRecord> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        let rows = sqlx::query_as::<_, AttendanceRecord>(
            "SELECT * FROM Attendance WHERE date = $1 AND member_id=$2",
        )
        .bind(date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(rows)
    }

    async fn present_count(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64> {
        if end_date < start_date {
            return Err("end_date must be >= start_date".into());
        }

        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let records: i64 = sqlx::query_scalar(
            "
        SELECT COUNT(att.is_present)
        FROM attendance att
        INNER JOIN member m ON att.member_id = m.member_id
        WHERE att.member_id = $3
          AND att.is_present = true
          AND att.date BETWEEN $1 AND $2",
        )
        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(records)
    }

    async fn absent_count(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64> {
        if end_date < start_date {
            return Err("end_date must be >= start_date".into());
        }

        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let working_days: i64 = sqlx::query_scalar(
            "
        SELECT COUNT(*) 
        FROM (
        SELECT date
        FROM attendance
        where date between $1 and $2 GROUP BY date
        HAVING BOOL_or(is_present = true)
        );
        ",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool.as_ref())
        .await?;

        let present: i64 = sqlx::query_scalar(
            "
        SELECT COUNT(att.is_present)
        FROM attendance att
        WHERE att.member_id = $3
          AND att.is_present = true
          AND att.date BETWEEN $1 AND $2",
        )
        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(working_days - present)
    }
}

#[ComplexObject]
impl Member {
    async fn status(&self, _ctx: &Context<'_>) -> StatusInfo {
        StatusInfo {
            member_id: self.member_id,
        }
    }

    async fn attendance(&self, _ctx: &Context<'_>) -> AttendanceInfo {
        AttendanceInfo {
            member_id: self.member_id,
        }
    }
}
