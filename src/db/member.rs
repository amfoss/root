use async_graphql::SimpleObject;
use sqlx::FromRow;

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
    pub leaderboard_id: String,
    pub cp_platform: String,
    pub rating:String,
}
