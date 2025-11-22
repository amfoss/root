use crate::models::member::Member;
use async_graphql::{Enum, SimpleObject};
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Enum, Copy, Clone, Eq, PartialEq, sqlx::Type, Debug)]
#[sqlx(type_name = "role_type")]
pub enum Role {
    Admin,
    Member,
    Bot,
}

#[derive(SimpleObject, FromRow, Debug)]
pub struct Session {
    pub session_id: i32,
    pub member_id: i32,
    #[graphql(skip)]
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    #[graphql(skip)]
    pub created_at: NaiveDateTime,
}

#[derive(SimpleObject, FromRow, Debug)]
pub struct ApiKey {
    pub api_key_id: i32,
    pub name: String,
    #[graphql(skip)]
    pub key_hash: String,
    pub created_by: Option<i32>,
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
}

// Response types for auth mutations
#[derive(SimpleObject)]
pub struct AuthResponse {
    pub member: Member,
    pub session_token: String,
}

#[derive(SimpleObject)]
pub struct ApiKeyResponse {
    pub api_key: String,
}

// OAuth callback data (not an input, used internally)
#[derive(Debug, Clone)]
pub struct GitHubUser {
    pub github_id: i64,
    pub github_username: String,
    pub name: String,
    pub email: String,
}
