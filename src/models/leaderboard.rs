use async_graphql::SimpleObject;
use sqlx::FromRow;

#[derive(FromRow, SimpleObject)]
pub struct Leaderboard {
    pub id: i32,
    pub member_id: i32,
    pub leetcode_score: Option<i32>,
    pub codeforces_score: Option<i32>,
    pub unified_score: i32,
    pub last_updated: Option<chrono::NaiveDateTime>,
}

#[derive(FromRow, SimpleObject)]
pub struct LeaderboardWithMember {
    pub id: i32,
    pub member_id: i32,
    pub member_name: String,
    pub leetcode_score: Option<i32>,
    pub codeforces_score: Option<i32>,
    pub unified_score: i32,
    pub last_updated: Option<chrono::NaiveDateTime>,
}

#[derive(FromRow, SimpleObject)]
pub struct LeetCodeStats {
    pub id: i32,
    pub member_id: i32,
    pub leetcode_username: String,
    pub problems_solved: i32,
    pub easy_solved: i32,
    pub medium_solved: i32,
    pub hard_solved: i32,
    pub contests_participated: i32,
    pub best_rank: i32,
    pub total_contests: i32,
}

#[derive(FromRow, SimpleObject)]
pub struct LeetCodeStatsWithName {
    pub id: i32,
    pub member_id: i32,
    pub member_name: String,
    pub leetcode_username: String,
    pub problems_solved: i32,
    pub easy_solved: i32,
    pub medium_solved: i32,
    pub hard_solved: i32,
    pub contests_participated: i32,
    pub best_rank: i32,
    pub total_contests: i32,
}

#[derive(FromRow, SimpleObject)]
pub struct CodeforcesStats {
    pub member_id: i32,
    pub codeforces_handle: String,
    pub codeforces_rating: i32,
    pub max_rating: i32,
    pub contests_participated: i32,
}

#[derive(FromRow, SimpleObject)]
pub struct CodeforcesStatsWithName {
    pub member_id: i32,
    pub member_name: String,
    pub codeforces_handle: String,
    pub codeforces_rating: i32,
    pub max_rating: i32,
    pub contests_participated: i32,
}
