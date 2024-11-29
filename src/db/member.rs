use sqlx::FromRow;
use async_graphql::SimpleObject;


//Struct for the Member table
#[derive(FromRow, SimpleObject)]

pub struct Member {
    pub id: i32,
    pub rollno: String,
    pub name: String,
    pub hostel: String,
    pub email: String,
    pub sex: String,
    pub year: i32,
    pub macaddress: String,
    pub discord_id: Option<String>,
    pub group_id: Option<i32>,
}

#[derive(FromRow, SimpleObject)]
pub struct StreakUpdate {
    pub id: i32,
    pub streak: Option<i32>,
    pub max_streak: Option<i32>,
}