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
        let status = sqlx::query_as::<_, StatusBreakRecord>(
            "INSERT INTO StatusBreaks (start_date, end_date, year, reason)
             VALUES ($1, $2, $3, $4)
             RETURNING *
            ",
        )
        .bind(input.start_date)
        .bind(input.end_date)
        .bind(input.year)
        .bind(&input.reason)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(status)
    }
}
