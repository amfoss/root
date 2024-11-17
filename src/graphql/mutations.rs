use async_graphql::{Context, Object};
use ::chrono::Local;
use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use sqlx::types::chrono;
use std::sync::Arc;
use hmac::{Hmac,Mac};
use sha2::Sha256;


type HmacSha256 = Hmac<Sha256>;

use crate::db::{member::Member, attendance::Attendance};

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
        leaderboard_id:String,
        cp_platform: String,

    ) -> Result<Member, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");



        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (rollno, name, hostel, email, sex, year, macaddress, leaderboard_id, cp_platform) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"
        )
        .bind(rollno)
        .bind(name)
        .bind(hostel)
        .bind(email)
        .bind(sex)
        .bind(year)
        .bind(macaddress)
        .bind(leaderboard_id)
        .bind(cp_platform)
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
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");


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
    ) -> Result<Attendance,sqlx::Error> {
        
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");

        let secret_key = ctx.data::<String>().expect("HMAC secret not found in context");

        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

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
            "
        )
        .bind(current_time)
        .bind(is_present)
        .bind(id)
        .bind(date)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(attendance)
    }
}
