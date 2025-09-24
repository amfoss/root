use crate::graphql::api::leaderboard_api::fetch_and_update_leetcode;
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct FetchLeetCode;

#[Object]
impl FetchLeetCode {
    pub async fn fetch_leetcode_stats(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        username: String,
    ) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        fetch_and_update_leetcode(pool.clone(), member_id, &username).await?;
        Ok(true)
    }
}
