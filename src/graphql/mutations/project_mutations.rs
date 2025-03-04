use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::models::project::{Project, SetProjectInput};

#[derive(Default)]
pub struct ProjectMutations;

#[Object]
impl ProjectMutations {
    #[graphql(name = "setProject")]
    async fn set_project(&self, ctx: &Context<'_>, input: SetProjectInput) -> Result<Project> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .expect("Pool must be found in context");

        let project = sqlx::query_as::<_, Project>(
            "INSERT INTO Project (member_id, project_title) VALUES ($1, $2) RETURNING * ",
        )
        .bind(input.member_id)
        .bind(input.title)
        .fetch_one(pool.as_ref())
        .await?;
        Ok(project)
    }
}
