use chrono::NaiveDate;
use std::sync::Arc;

use crate::models::{
    attendance::{Attendance, AttendanceWithMember},
    member::Member,
};
use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::models::{
    attendance::{Attendance, AttendanceReport, DailyCount, MemberAttendanceSummary},
    member::Member,
};

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

        if let Some(id) = member_id {
            let attendance_query =
                sqlx::query_as::<_, Attendance>("SELECT * FROM Attendance WHERE member_id = $1")
                    .bind(id)
                    .fetch_all(pool.as_ref())
                    .await?;

            return Ok(attendance_query);
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

        let attendance_query =
            sqlx::query_as::<_, Attendance>("SELECT * FROM Attendance WHERE member_id = $1")
                .bind(member.member_id)
                .fetch_all(pool.as_ref())
                .await?;

        Ok(attendance_query)
    }

    async fn get_attendance_summary(
        &self,
        ctx: &Context<'_>,
        start_date: String,
        end_date: String,
    ) -> Result<AttendanceReport> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|_| async_graphql::Error::new("Invalid start_date format. Use YYYY-MM-DD"))?;
        let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
            .map_err(|_| async_graphql::Error::new("Invalid end_date format. Use YYYY-MM-DD"))?;

        let daily_count_query = sqlx::query!(
            r#"
            WITH dates AS (
            SELECT generate_series($1::date, $2::date, '1 day') as day
            )
            SELECT dates.day as date,
                COUNT(a.member_id) as total_present
            FROM dates
            LEFT JOIN Attendance a ON dates.day = a.date AND a.is_present = true
            GROUP BY dates.day
            ORDER BY dates.day
            "#,
            start,
            end
        );

        let daily_count: Vec<DailyCount> = daily_count_query
            .fetch_all(pool.as_ref())
            .await?
            .into_iter()
            .map(|row| DailyCount {
                date: row.date.unwrap_or_default().to_string(),
                count: row.total_present.unwrap_or(0),
            })
            .collect();

        let member_attendance_query = sqlx::query!(
            r#"
            SELECT m.member_id as "id!", m.name as "name!",
                COUNT(a.is_present)::int as "present_days!"
            FROM Member m
            LEFT JOIN Attendance a 
                ON m.member_id = a.member_id 
                AND a.is_present AND a.date >= CURRENT_DATE - INTERVAL '6 months'
            GROUP BY m.member_id, m.name
            ORDER BY m.member_id
            "#
        )
        .fetch_all(pool.as_ref())
        .await?;

        let member_attendance = member_attendance_query
            .into_iter()
            .map(|row| MemberAttendanceSummary {
                id: row.id,
                name: row.name,
                present_days: row.present_days as i64,
            })
            .collect();

        let max_days = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT date) FROM Attendance 
            WHERE date >= CURRENT_DATE - INTERVAL '6 months' AND is_present",
        )
        .fetch_one(pool.as_ref())
        .await?;

        Ok(AttendanceReport {
            daily_count,
            member_attendance,
            max_days: max_days as i64,
        })
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
