use async_graphql::{ComplexObject, Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;
use chrono::NaiveDate;


use crate::models::{
    attendance::{AttendanceInfo, AttendanceSummaryInfo},
    member::Member,
    project::Project,
    status_update::StatusUpdateStreakInfo,
};

#[derive(Default)]
pub struct MemberQueries;

#[Object]
impl MemberQueries {
    pub async fn members(
        &self,
        ctx: &Context<'_>,
        year: Option<i32>,
        group_id: Option<i32>,
    ) -> Result<Vec<Member>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let mut query = sqlx::QueryBuilder::new("SELECT * FROM Member WHERE 1=1");

        if let Some(y) = year {
            query.push(" AND year = ");
            query.push_bind(y);
        }

        if let Some(g) = group_id {
            query.push(" AND group_id = ");
            query.push_bind(g);
        }

        let members = query
            .build_query_as::<Member>()
            .fetch_all(pool.as_ref())
            .await?;

        Ok(members)
    }
}

#[ComplexObject]
impl Member {
    async fn attendance(&self, ctx: &Context<'_>) -> Vec<AttendanceInfo> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        sqlx::query_as::<_, AttendanceInfo>(
            "SELECT date, is_present, time_in, time_out FROM Attendance WHERE member_id = $1",
        )
        .bind(self.member_id)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default()
    }

    #[graphql(name = "attendanceSummary")]
    async fn attendance_summary(&self, ctx: &Context<'_>) -> Vec<AttendanceSummaryInfo> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        sqlx::query_as::<_, AttendanceSummaryInfo>(
            "SELECT
                to_char(month_date, 'YYYY') AS year,
                to_char(month_date, 'MM') AS month,
                count(*) AS days_attended
            FROM (
                SELECT
                    date_trunc('month', date) AS month_date,
                    member_id
                FROM
                    attendance
                WHERE
                    is_present = TRUE
                    AND member_id = $1
            ) AS monthly_data
            GROUP BY
                month_date,
                member_id;",
        )
        .bind(self.member_id)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default()
    }

    async fn streak(&self, ctx: &Context<'_>) -> Vec<StatusUpdateStreakInfo> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        sqlx::query_as::<_, StatusUpdateStreakInfo>(
            "SELECT current_streak, max_streak FROM StatusUpdateStreak WHERE member_id = $1",
        )
        .bind(self.member_id)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default()
    }

    async fn projects(&self, ctx: &Context<'_>) -> Vec<Project> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        sqlx::query_as::<_, Project>("SELECT project_id, title FROM Project WHERE member_id = $1")
            .bind(self.member_id)
            .fetch_all(pool.as_ref())
            .await
            .unwrap_or_default()


    }

    async fn present_count_by_date(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date:NaiveDate,
    ) -> Result<i64>{

        if end_date < start_date {
            return Err("end_date must be >= start_date".into());
        }

        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let records : i64 = sqlx::query_scalar(
        "
        SELECT COUNT(att.is_present)
        FROM attendance att
        INNER JOIN member m ON att.member_id = m.member_id
        WHERE att.member_id = $3
          AND att.is_present = true
          AND att.date BETWEEN $1 AND $2"
        )
        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(records)

    }

    async fn absent_count_by_date(
        &self,
        ctx: &Context<'_>,
        start_date: NaiveDate,
        end_date:NaiveDate,
    ) -> Result<i64>{

        if end_date < start_date {
            return Err("end_date must be >= start_date".into());
        }

        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let working_days : i64 = sqlx::query_scalar(
        "
        SELECT COUNT(*) AS working_days
        FROM (
        SELECT date
        FROM attendance
        where date between $1 and $2 GROUP BY date
        HAVING BOOL_or(is_present = true)
        ) AS working_dates;
        "
        )

        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        let present : i64 = sqlx::query_scalar(
        "
        SELECT COUNT(att.is_present)
        FROM attendance att
        INNER JOIN member m ON att.member_id = m.member_id
        WHERE att.member_id = $3
          AND att.is_present = true
          AND att.date BETWEEN $1 AND $2"
        )
        .bind(start_date)
        .bind(end_date)
        .bind(self.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(working_days-present)

    }
}
