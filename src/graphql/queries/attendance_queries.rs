use std::sync::Arc;

use crate::models::{
    attendance::{Attendance, AttendanceWithMember},
    member::Member,
};
use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use sqlx::PgPool;

/// Sub-query for the [`Attendance`] table. The queries are:
/// * attendance - get a specific member's attendance details using their member_id, roll_no or discord_id, or by date for all members.
#[derive(Default)]
pub struct AttendanceQueries;

#[Object]
impl AttendanceQueries {
    async fn attendance(
        &self,
        ctx: &Context<'_>,
        member_id: Option<i32>,
        roll_no: Option<String>,
        discord_id: Option<String>,
    ) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        // member_id is given, simple query
        if let Some(id) = member_id {
            let attendance_query =
                sqlx::query_as::<_, Attendance>("SELECT * FROM Attendance WHERE member_id = $1")
                    .bind(id)
                    .fetch_all(pool.as_ref())
                    .await?;

            return Ok(attendance_query);
        }

        // Get the member using their roll_no or discord_id
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

        let attendance_query =
            sqlx::query_as::<_, Attendance>("SELECT * FROM Attendance WHERE member_id = $1")
                .bind(member.member_id)
                .fetch_all(pool.as_ref())
                .await?;

        Ok(attendance_query)
    }

    // Query to get attendance by date
    async fn attendance_by_date(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> Result<Vec<AttendanceWithMember>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let records = sqlx::query_as::<_, AttendanceWithMember>(
            "SELECT a.attendance_id, a.member_id, a.date, a.is_present, 
                    a.time_in, a.time_out, m.name, m.year
             FROM Attendance a
             JOIN Member m ON a.member_id = m.member_id
             WHERE a.date = $1",
        )
        .bind(date)
        .fetch_all(pool.as_ref())
        .await?;

        Ok(records)
    }
}
