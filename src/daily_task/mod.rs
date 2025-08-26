use chrono::NaiveTime;
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::sleep_until;
use tracing::{debug, error, info};

use crate::models::member::Member;

pub async fn run_daily_task_at_midnight(pool: Arc<PgPool>) {
    loop {
        let now = chrono::Utc::now().with_timezone(&Kolkata);
        let naive_midnight =
            NaiveTime::from_hms_opt(00, 30, 00).expect("Hardcoded time must be valid");
        let today_midnight = now
            .with_time(naive_midnight)
            .single()
            .expect("Hardcoded time must be valid");

        let next_midnight = if now >= today_midnight {
            today_midnight + chrono::Duration::days(1)
        } else {
            today_midnight
        };
        debug!("next_midnight: {}", next_midnight);

        let duration_until_midnight = next_midnight.signed_duration_since(now);
        info!("Sleeping for {}", duration_until_midnight.num_seconds());
        let sleep_duration =
            tokio::time::Duration::from_secs(duration_until_midnight.num_seconds() as u64);

        sleep_until(tokio::time::Instant::now() + sleep_duration).await;
        execute_daily_task(pool.clone()).await;
    }
}

/// This function does a number of things, including:
/// * Insert new attendance records everyday for [`presense`](https://www.github.com/amfoss/presense) to update them later in the day.
async fn execute_daily_task(pool: Arc<PgPool>) {
    // Members is queried outside of each function to avoid repetition
    let members = sqlx::query_as::<_, Member>("SELECT * FROM Member")
        .fetch_all(&*pool)
        .await;

    match members {
        Ok(members) => {
            update_attendance(&members, &pool).await;
            update_status_history(&members, &pool).await;
        }
        // TODO: Handle this
        Err(e) => error!("Failed to fetch members: {:?}", e),
    };
}

async fn update_attendance(members: &Vec<Member>, pool: &PgPool) {
    #[allow(deprecated)]
    let today = chrono::Utc::now()
        .with_timezone(&Kolkata)
        .date()
        .naive_local();
    debug!("Updating attendance on {}", today);

    for member in members {
        let attendance = sqlx::query(
            "INSERT INTO Attendance (member_id, date, is_present, time_in, time_out)
                     VALUES ($1, $2, $3, $4, $5)
                     ON CONFLICT (member_id, date) DO NOTHING",
        )
        .bind(member.member_id)
        .bind(today)
        .bind(false)
        .bind(None::<NaiveTime>)
        .bind(None::<NaiveTime>)
        .execute(pool)
        .await;

        match attendance {
            Ok(_) => {
                debug!(
                    "Attendance record added for member ID: {}",
                    member.member_id
                );
            }
            Err(e) => {
                error!(
                    "Failed to insert attendance for member ID: {}: {:?}",
                    member.member_id, e
                );
            }
        }
        // This could have been called in `execute_daily_task()` but that would require us to loop through members twice.
    }
}

async fn update_status_history(members: &Vec<Member>, pool: &PgPool) {
    #[allow(deprecated)]
    let today = chrono::Utc::now()
        .with_timezone(&Kolkata)
        .date()
        .naive_local();
    debug!("Updating Status Update History on {}", today);

    for member in members {
        let status_update = sqlx::query(
            "INSERT INTO StatusUpdateHistory (member_id, date, is_updated) 
                     VALUES ($1, $2, $3)
                     ON CONFLICT (member_id, date) DO NOTHING",
        )
        .bind(member.member_id)
        .bind(today)
        .bind(false)
        .execute(pool)
        .await;

        match status_update {
            Ok(_) => {
                debug!(
                    "Status update record added for member ID: {}",
                    member.member_id
                );
            }
            Err(e) => {
                error!(
                    "Failed to insert status update history for member ID: {}: {:?}",
                    member.member_id, e
                );
            }
        }
    }
}
