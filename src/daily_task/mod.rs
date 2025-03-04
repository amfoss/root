use chrono::{Datelike, NaiveDate, NaiveTime};
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::sleep_until;
use tracing::{debug, error, info};

use crate::models::member::Member;

/// Continuously sleep till midnight, then run the 'execute_daily_task' function.
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
/// * Update the AttendanceSummary table
async fn execute_daily_task(pool: Arc<PgPool>) {
    // Members is queried outside of each function to avoid repetition
    let members = sqlx::query_as::<_, Member>("SELECT * FROM Member")
        .fetch_all(&*pool)
        .await;

    match members {
        // Add additional daily tasks such as leaderboard updates to the Ok(members) arm
        Ok(members) => update_attendance(members, &pool).await,
        // TODO: Handle this
        Err(e) => error!("Failed to fetch members: {:?}", e),
    };
}

// We need to add a record for every member because otherwise [`Presense`](https://www.github.com/presense) will only add present members to the DB, and we will have to JOIN Members and Attendance records for the day to get the absent members. In exchange for increased storage use, we get simpler queries for Home which needs the data for every member for every day so far. But as of Jan 2025, there are less than 50 members in the club and thus storage really shouldn't be an issue.
/// Inserts new attendance records everyday for [`presense`](https://www.github.com/amfoss/presense) to update them later in the day and updates the AttendanceSummary table to keep track of monthly streaks.
async fn update_attendance(members: Vec<Member>, pool: &PgPool) {
    #[allow(deprecated)]
    let today = chrono::Utc::now()
        .with_timezone(&Kolkata)
        .date()
        .naive_local();
    debug!("Updating attendance on {}", today);

    for member in members {
        // Insert blank rows for each member
        let attendance = sqlx::query(
            "INSERT INTO Attendance (member_id, date, is_present, time_in, time_out) 
                     VALUES ($1, $2, $3, $4, $5)
                     ON CONFLICT (member_id, date) DO NOTHING",
        )
        .bind(member.member_id)
        .bind(today)
        .bind(false) // Default `is_present` is False
        .bind(None::<NaiveTime>) // Default `time_in` is NULL
        .bind(None::<NaiveTime>) // Default `time_out` is NULL
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
        // Whether or not inserting attendance failed, Root will attempt to update AttendanceSummary. This can potentially fail too since insertion failed earlier. However, these two do not depend on each other and one of them failing is no reason to avoid trying the other.
        update_attendance_summary(member.member_id, pool).await;
    }
}

/// Checks if the member was present yesterday, and if so, increments the `days_attended` value. Otherwise, do nothing.
async fn update_attendance_summary(member_id: i32, pool: &PgPool) {
    debug!("Updating summary for member #{}", member_id);
    #[allow(deprecated)]
    let today = chrono::Utc::now()
        .with_timezone(&Kolkata)
        .date()
        .naive_local();
    let yesterday = today - chrono::Duration::days(1);

    // Check if the member was present yesterday
    let was_present_yesterday = sqlx::query_scalar::<_, bool>(
        r#"
            SELECT is_present 
            FROM Attendance 
            WHERE member_id = $1 AND date = $2
        "#,
    )
    .bind(member_id)
    .bind(yesterday)
    .fetch_one(pool)
    .await;

    match was_present_yesterday {
        // Member was present yesterday, update the summary
        Ok(true) => {
            update_days_attended(member_id, today, pool).await;
        }
        // Member was absent
        Ok(false) => {
            debug!(
                "Member ID: {} was absent yesterday, days_attended remains the same.",
                member_id
            );
        }
        Err(e) => {
            error!("Could not fetch records from DB. Error: {}", e);
        }
    }
}

/// Increments the `days_attended` value for the given member in the given month.
async fn update_days_attended(member_id: i32, today: NaiveDate, pool: &PgPool) {
    // Convert year and month into i32 cause SQLx cannot encode u32 into database types
    let month: i32 = (today.month0() + 1) as i32;
    let year: i32 = today.year_ce().1 as i32;

    // Check if there's an existing summary for the current month
    let existing_days_attended = sqlx::query_scalar::<_, i32>(
        r#"
            SELECT days_attended
            FROM AttendanceSummary
            WHERE member_id = $1
            AND year = $2
            AND month = $3
        "#,
    )
    .bind(member_id)
    .bind(year)
    .bind(month)
    .fetch_optional(pool)
    .await;

    match existing_days_attended {
        Ok(Some(days_attended)) => {
            sqlx::query(
                r#"
                    UPDATE AttendanceSummary
                    SET days_attended = days_attended + 1
                    WHERE member_id = $2
                    AND year = $3
                    AND month = $4
                "#,
            )
            .bind(member_id)
            .bind(year)
            .bind(month)
            .execute(pool)
            .await
            .unwrap();

            debug!(
                "Updated days_attended for member ID: {}. New days_attended: {}",
                member_id,
                days_attended + 1
            );
        }
        // No summary exists for this month, create a new one
        Ok(None) => {
            sqlx::query(
                r#"
                    INSERT INTO AttendanceSummary (member_id, year, month, days_attended)
                    VALUES ($1, $2, 1)
                "#,
            )
            .bind(member_id)
            .bind(year)
            .bind(month)
            .execute(pool)
            .await
            .unwrap();

            debug!(
                "Created new streak for member ID: {} for the month.",
                member_id
            );
        }
        Err(e) => {
            error!(
                "Error checking or updating streak for member ID {}: {:?}",
                member_id, e
            );
        }
    }
}
