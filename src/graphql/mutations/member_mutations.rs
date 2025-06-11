use crate::models::member::{CreateMemberInput, Member, UpdateMemberInput};
use async_graphql::{Context, Object, Result};
use chrono::Local;
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct MemberMutations;

#[Object]
impl MemberMutations {
    #[graphql(name = "createMember")]
    async fn create_member(&self, ctx: &Context<'_>, input: CreateMemberInput) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");
        let now = Local::now().with_timezone(&Kolkata).date_naive();

        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (roll_no, name, email, sex, year, hostel, mac_address, discord_id, group_id, track, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"
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
        .bind(&input.track)
        .bind(now)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }

    #[graphql(name = "updateMember")]
    async fn update_member(&self, ctx: &Context<'_>, input: UpdateMemberInput) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let member = sqlx::query_as::<_, Member>(
            "UPDATE Member SET
                roll_no = COALESCE($1, roll_no),
                name = COALESCE($2, name),
                email = COALESCE($3, email),
                sex = COALESCE($4, sex),
                year = COALESCE($5, year),
                hostel = COALESCE($6, hostel),
                mac_address = COALESCE($7, mac_address),
                discord_id = COALESCE($8, discord_id),
                group_id = COALESCE($9, group_id),
                track = COALESCE($10, track)
            WHERE member_id = $11
            RETURNING *",
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
        .bind(&input.track)
        .bind(input.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }
}
