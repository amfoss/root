use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::Local;
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;

use crate::models::member::{CreateMemberInput, Member};

#[derive(Default)]
pub struct MemberMutations;

#[Object]
impl MemberMutations {
    #[graphql(name = "createMember")]
    async fn create_member(&self, ctx: &Context<'_>, input: CreateMemberInput) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let now = Local::now().with_timezone(&Kolkata).date_naive();
        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (roll_no, name, email, sex, year, hostel, mac_address, discord_id, group_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"
        )
        .bind(&input.roll_no)
        .bind(&input.name)
        .bind(&input.email)
        .bind(input.sex)
        .bind(input.year)
        .bind(&input.hostel)
        .bind(&input.mac_address)
        .bind(&input.discord_id)
        .bind(input.group_id)
        .bind(now)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }
}
