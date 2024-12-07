use chrono::{Local, NaiveTime};
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    db::{
        leaderboard::{CodeforcesStats, LeetCodeStats},
        member::Member,
    },
    leaderboard::{
        fetch_stats::{fetch_codeforces_stats, fetch_leetcode_stats},
        update_leaderboard::update_leaderboard,
    },
};
//Scheduled task for moving all members to Attendance table at midnight.
pub async fn scheduled_task(pool: Arc<PgPool>) {
    let members: Result<Vec<Member>, sqlx::Error> =
        sqlx::query_as::<_, Member>("SELECT * FROM Member")
            .fetch_all(pool.as_ref())
            .await;

    match members {
        Ok(members) => {
            let today = Local::now().with_timezone(&Kolkata);

            for member in members {
                let timein = NaiveTime::from_hms_opt(0, 0, 0);
                let timeout = NaiveTime::from_hms_opt(0, 0, 0); // Default time, can be modified as needed

                let attendance = sqlx::query(
                    "INSERT INTO Attendance (id, date, timein, timeout, is_present) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id, date) DO NOTHING RETURNING *"
                )
                .bind(member.id)
                .bind(today)
                .bind(timein)
                .bind(timeout)
                .bind(false)
                .execute(pool.as_ref())
                .await;

                match attendance {
                    Ok(_) => println!("Attendance record added for member ID: {}", member.id),
                    Err(e) => eprintln!(
                        "Failed to insert attendance for member ID: {}: {:?}",
                        member.id, e
                    ),
                }

                //fetching the username from tables
                let leetcode_username = sqlx::query_as::<_, LeetCodeStats>(
                    "SELECT * FROM leetcode_stats WHERE member_id = $1",
                )
                .bind(member.id)
                .fetch_optional(pool.as_ref())
                .await;

                if let Ok(Some(leetcode_stats)) = leetcode_username {
                    let username = leetcode_stats.leetcode_username.clone();
                    
                    // Fetch and update LeetCode stats
                    match fetch_leetcode_stats(pool.clone(), member.id, &username).await {
                        Ok(_) => println!("LeetCode stats updated for member ID: {}", member.id),
                        Err(e) => eprintln!(
                            "Failed to update LeetCode stats for member ID {}: {:?}",
                            member.id, e
                        ),
                    }
                }

                // Fetch Codeforces username
                let codeforces_username = sqlx::query_as::<_, CodeforcesStats>(
                    "SELECT * FROM codeforces_stats WHERE member_id = $1",
                )
                .bind(member.id)
                .fetch_optional(pool.as_ref())
                .await;

                if let Ok(Some(codeforces_stats)) = codeforces_username {
                    let username = codeforces_stats.codeforces_handle.clone();

                    // Fetch and update Codeforces stats
                    match fetch_codeforces_stats(pool.clone(), member.id, &username).await {
                        Ok(_) => println!("Codeforces stats updated for member ID: {}", member.id),
                        Err(e) => eprintln!(
                            "Failed to update Codeforces stats for member ID {}: {:?}",
                            member.id, e
                        ),
                    }
                }

                match update_leaderboard(pool.clone()).await {
                    Ok(_) => println!("Leaderboard updated."),
                    Err(e) => eprintln!("Failed to update leaderboard: {:?}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch members: {:?}", e),
    }
}
