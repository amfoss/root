use sqlx::FromRow;
use async_graphql::SimpleObject;

#[derive(FromRow, SimpleObject)]
pub struct ActiveProjects {
    id: i32,
    member_id: i32,
    project_title: Option<String>,
}