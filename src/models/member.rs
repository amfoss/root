use crate::models::auth::Role;
use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Enum, Copy, Clone, Eq, PartialEq, sqlx::Type, Debug)]
#[sqlx(type_name = "sex_type")]
pub enum Sex {
    M,
    F,
    Other,
}

#[derive(SimpleObject, FromRow, Clone, Debug)]
#[graphql(complex)]
pub struct Member {
    pub member_id: i32,
    pub roll_no: Option<String>,
    pub name: String,
    pub email: String,
    pub sex: Option<Sex>,
    pub year: Option<i32>,
    pub hostel: Option<String>,
    pub mac_address: Option<String>,
    pub discord_id: Option<String>,
    pub group_id: Option<i32>,
    pub role: Role,
    pub track: Option<String>,
    pub github_user: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(InputObject)]
pub struct CreateMemberInput {
    pub roll_no: String,
    pub name: String,
    pub email: String,
    pub sex: Sex,
    pub year: i32,
    pub hostel: String,
    pub mac_address: String,
    pub discord_id: String,
    pub group_id: i32,
    pub track: Option<String>,
    pub github_user: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateMemberInput {
    pub roll_no: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub sex: Option<Sex>,
    pub year: Option<i32>,
    pub hostel: Option<String>,
    pub mac_address: Option<String>,
    pub discord_id: Option<String>,
    pub group_id: Option<i32>,
    pub track: Option<String>,
    pub github_user: Option<String>,
}
