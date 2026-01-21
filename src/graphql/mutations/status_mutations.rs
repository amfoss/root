use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::guards::{AdminGuard, AdminOrBotGuard};
use crate::models::status_update::{CreateStatusBreakInput, StatusBreakRecord, StatusUpdateRecord};

#[derive(Default)]
pub struct StatusMutations;

#[Object]
impl StatusMutations {
    #[graphql(name = "markStatusUpdate", guard = "AdminOrBotGuard")]
    async fn mark_status_update(
        &self,
        ctx: &Context<'_>,
        emails: Vec<String>,
        date: NaiveDate,
    ) -> Result<Vec<StatusUpdateRecord>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context");
        #[allow(deprecated)]
        let status = sqlx::query_as::<_, StatusUpdateRecord>(
            "UPDATE StatusUpdateHistory SET
                is_sent = true
            WHERE member_id IN (SELECT member_id from Member where email = ANY($1))
            AND date = $2
            RETURNING *
            ",
        )
        .bind(emails)
        .bind(date)
        .fetch_all(pool.as_ref())
        .await?;

        Ok(status)
    }

    #[graphql(name = "createStatusBreak", guard = "AdminGuard")]
    async fn create_status_break(
        &self,
        ctx: &Context<'_>,
        input: CreateStatusBreakInput,
    ) -> Result<StatusBreakRecord> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context");

        match (&input.year, &input.member_id) {
            (Some(_), Some(_)) => {
                return Err("Cannot specify both year and member_id. A status break must apply to either a year or a member, not both.".into());
            }
            (None, None) => {
                return Err("Must specify either year or member_id. A status break must apply to either a year or a member.".into());
            }
            _ => {}
        }

        if input.start_date >= input.end_date {
            return Err("start_date must be before end_date".into());
        }

        let status_break = sqlx::query_as::<_, StatusBreakRecord>(
            "INSERT INTO StatusBreaks (start_date, end_date, year, member_id, reason)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *
            ",
        )
        .bind(input.start_date)
        .bind(input.end_date)
        .bind(input.year)
        .bind(input.member_id)
        .bind(&input.reason)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(status_break)
    }
}
