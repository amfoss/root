use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::Local;
use chrono_tz::Asia::Kolkata;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;

use crate::models::attendance::{AttendanceRecord, MarkAttendanceInput};

type HmacSha256 = Hmac<Sha256>;

#[derive(Default)]
pub struct AttendanceMutations;

#[Object]
impl AttendanceMutations {
    #[graphql(name = "markAttendance")]
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

        let now = Local::now().with_timezone(&Kolkata).time();
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
}
