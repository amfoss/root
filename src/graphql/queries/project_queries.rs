use std::sync::Arc;

use crate::models::project::Project;
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

#[derive(Default)]
pub struct ProjectQueries;

#[Object]
impl ProjectQueries {
    pub async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let projects = sqlx::query_as::<_, Project>("SELECT * FROM Project")
            .fetch_all(pool.as_ref())
            .await?;

        Ok(projects)
    }
}
