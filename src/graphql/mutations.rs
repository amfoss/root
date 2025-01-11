use ::chrono::Local;
use async_graphql::{Context, Object};
use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Asia::Kolkata;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::types::chrono;
use sqlx::PgPool;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

use crate::models::{
    attendance::Attendance,
    leaderboard::{CodeforcesStats, LeetCodeStats},
    member::Member,
    member::StreakUpdate,
    projects::ActiveProjects,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    //Mutation for adding members to the Member table
    async fn add_member(
        &self,
        ctx: &Context<'_>,
        rollno: String,
        name: String,
        hostel: String,
        email: String,
        sex: String,
        year: i32,
        macaddress: String,
        discord_id: String,
        group_id: i32,
    ) -> Result<Member, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (rollno, name, hostel, email, sex, year, macaddress, discord_id, group_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"
        )
        .bind(rollno)
        .bind(name)
        .bind(hostel)
        .bind(email)
        .bind(sex)
        .bind(year)
        .bind(macaddress)
        .bind(discord_id)
        .bind(group_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }

    async fn edit_member(
        &self,
        ctx: &Context<'_>,
        id: i32,
        hostel: String,
        year: i32,
        macaddress: String,
        discord_id: String,
        group_id: i32,
        hmac_signature: String,
    ) -> Result<Member, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let secret_key = ctx
            .data::<String>()
            .expect("HMAC secret not found in context");

        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");

        let message = format!(
            "{}{}{}{}{}{}",
            id, hostel, year, macaddress, discord_id, group_id
        );
        mac.update(message.as_bytes());

        let expected_signature = mac.finalize().into_bytes();

        // Convert the received HMAC signature from the client to bytes for comparison
        let received_signature = hex::decode(hmac_signature)
            .map_err(|_| sqlx::Error::Protocol("Invalid HMAC signature".into()))?;

        if expected_signature.as_slice() != received_signature.as_slice() {
            return Err(sqlx::Error::Protocol("HMAC verification failed".into()));
        }

        let member = sqlx::query_as::<_, Member>(
            "
            UPDATE Member
            SET
                hostel = CASE WHEN $1 = '' THEN hostel ELSE $1 END,
                year = CASE WHEN $2 = 0 THEN year ELSE $2 END,
                macaddress = CASE WHEN $3 = '' THEN macaddress ELSE $3 END,
                discord_id = CASE WHEN $4 = '' THEN discord_id ELSE $4 END,
                group_id = CASE WHEN $5 = 0 THEN group_id ELSE $5 END
            WHERE id = $6
            RETURNING *
            ",
        )
        .bind(hostel)
        .bind(year)
        .bind(macaddress)
        .bind(discord_id)
        .bind(group_id)
        .bind(id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }

    //Mutation for adding attendance to the Attendance table
    async fn add_attendance(
        &self,

        ctx: &Context<'_>,
        id: i32,
        date: NaiveDate,
        timein: NaiveTime,
        timeout: NaiveTime,
        is_present: bool,
    ) -> Result<Attendance, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let attendance = sqlx::query_as::<_, Attendance>(
            "INSERT INTO Attendance (id, date, timein, timeout, is_present) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(id)
        .bind(date)
        .bind(timein)
        .bind(timeout)
        .bind(is_present)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(attendance)
    }

    async fn mark_attendance(
        &self,
        ctx: &Context<'_>,
        id: i32,
        date: NaiveDate,
        is_present: bool,
        hmac_signature: String,
    ) -> Result<Attendance, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let secret_key = ctx
            .data::<String>()
            .expect("HMAC secret not found in context");

        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");

        let message = format!("{}{}{}", id, date, is_present);
        mac.update(message.as_bytes());

        let expected_signature = mac.finalize().into_bytes();

        // Convert the received HMAC signature from the client to bytes for comparison
        let received_signature = hex::decode(hmac_signature)
            .map_err(|_| sqlx::Error::Protocol("Invalid HMAC signature".into()))?;

        if expected_signature.as_slice() != received_signature.as_slice() {
            return Err(sqlx::Error::Protocol("HMAC verification failed".into()));
        }

        let current_time = Local::now().with_timezone(&Kolkata).time();

        let attendance = sqlx::query_as::<_, Attendance>(
            "
            UPDATE Attendance
            SET 
                timein = CASE WHEN timein = '00:00:00' THEN $1 ELSE timein END,
                timeout = $1,
                is_present = $2
            WHERE id = $3 AND date = $4
            RETURNING *
            ",
        )
        .bind(current_time)
        .bind(is_present)
        .bind(id)
        .bind(date)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(attendance)
    }

    //here when user changes the handle, it just updates the handle in the database without updating the other values till midnight

    async fn add_or_update_leetcode_username(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        username: String,
    ) -> Result<LeetCodeStats, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let result = sqlx::query_as::<_, LeetCodeStats>(
            "
            INSERT INTO leetcode_stats (member_id, leetcode_username, problems_solved, easy_solved, medium_solved, hard_solved, contests_participated, best_rank, total_contests)
            VALUES ($1, $2, 0, 0, 0, 0, 0, 0, 0)
            ON CONFLICT (member_id) DO UPDATE
            SET leetcode_username = $2
            RETURNING *
            "
        )
        .bind(member_id)
        .bind(username)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(result)
    }

    async fn add_or_update_codeforces_handle(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        handle: String,
    ) -> Result<CodeforcesStats, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let result = sqlx::query_as::<_, CodeforcesStats>(
            "
            INSERT INTO codeforces_stats (member_id, codeforces_handle, codeforces_rating, max_rating, contests_participated)
            VALUES ($1, $2, 0, 0, 0)
            ON CONFLICT (member_id) DO UPDATE
            SET codeforces_handle = $2
            RETURNING *
            "
        )
        .bind(member_id)
        .bind(handle)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(result)
    }
    async fn update_streak(
        &self,
        ctx: &Context<'_>,
        id: i32,
        has_sent_update: bool,
    ) -> Result<StreakUpdate, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let streak_info = sqlx::query_as::<_, StreakUpdate>(
            "
            SELECT id, streak, max_streak
            FROM StreakUpdate
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

        match streak_info {
            Some(mut member) => {
                let current_streak = member.streak.unwrap_or(0);
                let max_streak = member.max_streak.unwrap_or(0);
                let (new_streak, new_max_streak) = if has_sent_update {
                    let updated_streak = current_streak + 1;
                    let updated_max_streak = updated_streak.max(max_streak);
                    (updated_streak, updated_max_streak)
                } else {
                    (0, max_streak)
                };
                let updated_member = sqlx::query_as::<_, StreakUpdate>(
                    "
                    UPDATE StreakUpdate
                    SET streak = $1, max_streak = $2
                    WHERE id = $3
                    RETURNING *
                    ",
                )
                .bind(new_streak)
                .bind(new_max_streak)
                .bind(id)
                .fetch_one(pool.as_ref())
                .await?;

                Ok(updated_member)
            }
            None => {
                let new_member = sqlx::query_as::<_, StreakUpdate>(
                    "
                    INSERT INTO StreakUpdate (id, streak, max_streak)
                    VALUES ($1, $2, $3)
                    RETURNING *
                    ",
                )
                .bind(id)
                .bind(0)
                .bind(0)
                .fetch_one(pool.as_ref())
                .await?;

                Ok(new_member)
            }
        }
    }

    async fn set_active_project(
        &self,
        ctx: &Context<'_>,
        id: i32,
        project_name: String,
    ) -> Result<ActiveProjects, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let active_project = sqlx::query_as::<_, ActiveProjects>(
            "
            INSERT INTO ActiveProjects (member_id,project_title)
            VALUES ($1,$2)
            RETURNING *
            ",
        )
        .bind(id)
        .bind(project_name)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(active_project)
    }

    async fn remove_active_project(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<ActiveProjects, sqlx::Error> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool not found in context");

        let active_project = sqlx::query_as::<_, ActiveProjects>(
            "
            DELETE FROM ActiveProjects
            WHERE id = $1
            RETURNING *
            ",
        )
        .bind(project_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(active_project)
    }
}
