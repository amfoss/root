use async_graphql::{ComplexObject, Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::{
    attendance::{AttendanceInfo, AttendanceSummaryInfo},
    member::Member,
    project::Project,
    status_update_streak::StatusUpdateStreakInfo,
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
            "SELECT year, month, days_attended FROM AttendanceSummary WHERE member_id = $1",
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
}
