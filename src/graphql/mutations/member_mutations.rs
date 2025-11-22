use crate::auth::guards::AuthGuard;
use crate::auth::AuthContext;
use crate::models::member::{Member, UpdateMemberInput};
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct MemberMutations;

#[Object]
impl MemberMutations {
    /// Update the details of the currently logged in member
    #[graphql(name = "updateMe", guard = "AuthGuard")]
    async fn update_me(&self, ctx: &Context<'_>, input: UpdateMemberInput) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");
        let auth = ctx
            .data::<AuthContext>()
            .expect("AuthContext must be in context.");

        let logged_in_user = auth.user.as_ref().ok_or("User not found in context")?;

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
                track = COALESCE($10, track),
                github_user = COALESCE($11, github_user)
            WHERE member_id = $12
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
        .bind(&input.github_user)
        .bind(logged_in_user.member_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }
}
