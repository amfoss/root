use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use chrono_tz::Asia::Kolkata;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;

use crate::auth::guards::AdminOrBotGuard;
use crate::models::attendance::{AttendanceRecord, LeaveRecord, MarkAttendanceInput};

type HmacSha256 = Hmac<Sha256>;

#[derive(Default)]
pub struct AttendanceMutations;

#[Object]
impl AttendanceMutations {
    #[graphql(name = "markAttendance", guard = "AdminOrBotGuard")]
    async fn mark_attendance(
        &self,
        ctx: &Context<'_>,
        input: MarkAttendanceInput,
    ) -> Result<AttendanceRecord> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let secret_key = ctx
            .data::<String>()
            .expect("ROOT_SECRET must be found in context");

        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        let message = format!("{}{}", input.member_id, input.date);
        mac.update(message.as_bytes());

        let expected_signature = mac.finalize().into_bytes();
        let received_signature = hex::decode(input.hmac_signature)?;

        if expected_signature.as_slice() != received_signature.as_slice() {
            return Err(async_graphql::Error::new("HMAC verification failed"));
        }

        let now = chrono::Utc::now().with_timezone(&Kolkata);

        let attendance = sqlx::query_as::<_, AttendanceRecord>(
            "UPDATE Attendance SET time_in = CASE 
                WHEN time_in IS NULL THEN $1 
                ELSE time_in END,
             time_out = $1,
             is_present = TRUE
             WHERE member_id = $2 AND date = $3 RETURNING *
            ",
        )
        .bind(now)
        .bind(input.member_id)
        .bind(input.date)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(attendance)
    }

    #[graphql(name = "leaveApplication", guard = "AdminOrBotGuard")]
    async fn leave_application(
        &self,
        ctx: &Context<'_>,
        discord_id: String,
        message_id: String,
        reason: String,
        from_date: NaiveDate,
        duration: i32,
    ) -> Result<LeaveRecord> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let leave: LeaveRecord = sqlx::query_as::<_, LeaveRecord>(
            "INSERT INTO Leave
            (discord_id, message_id, reason, from_date, duration) 
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            ",
        )
        .bind(discord_id)
        .bind(message_id)
        .bind(reason)
        .bind(from_date)
        .bind(duration)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(leave)
    }

    #[graphql(name = "approveLeave", guard = "AdminOrBotGuard")]
    async fn approve_leave(
        &self,
        ctx: &Context<'_>,
        discord_id: String,
        from_date: NaiveDate,
        approved_by: String,
    ) -> Result<LeaveRecord> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let leave: LeaveRecord = sqlx::query_as::<_, LeaveRecord>(
            "UPDATE Leave
            SET approved_by = $1
            WHERE discord_id = $2 AND
            from_date=$3 AND
            approved_by IS NULL
            RETURNING *
            ",
        )
        .bind(approved_by)
        .bind(discord_id)
        .bind(from_date)
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or_else(|| async_graphql::Error::new("no pending leave found to approve"))?;

        Ok(leave)
    }
}
