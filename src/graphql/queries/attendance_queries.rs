use crate::models::attendance::Attendance;
use crate::models::member::Member;
use async_graphql::{ComplexObject, Context, Object, Result};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct AttendanceQueries;

#[ComplexObject]
impl Attendance {
    async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let member = sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE member_id = $1")
            .bind(self.member_id)
            .fetch_one(pool.as_ref())
            .await?;

        Ok(member)
    }
}

#[Object]
impl AttendanceQueries {
    async fn attendance_by_date(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        let rows = sqlx::query_as::<_, Attendance>("SELECT * FROM Attendance WHERE date = $1")
            .bind(date)
            .fetch_all(pool.as_ref())
            .await?;

        Ok(rows)
    }
}
