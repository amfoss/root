use crate::auth::oauth::GitHubOAuthService;
use crate::models::auth::{GitHubUser, Role};
use crate::models::member::Member;
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;

pub struct AuthService;

impl AuthService {
    pub async fn handle_github_callback(pool: &PgPool, code: String) -> Result<Member, String> {
        let oauth_service = GitHubOAuthService::new()
            .map_err(|e| format!("Failed to initialize OAuth service: {}", e))?;

        let github_user = oauth_service
            .complete_oauth_flow(code)
            .await
            .map_err(|e| format!("OAuth flow failed: {}", e))?;

        let existing_member = sqlx::query_as::<_, Member>(
            "SELECT member_id, roll_no, name, email, sex, year, hostel, mac_address, discord_id,
             group_id, track, github_user, role, created_at, updated_at
             FROM Member
             WHERE github_user = $1",
        )
        .bind(&github_user.github_username)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to query member: {}", e))?;

        let member = if let Some(member) = existing_member {
            // Member exists - return existing member
            member
        } else {
            // Member doesn't exist - register new member
            Self::register_member(pool, github_user).await?
        };

        Ok(member)
    }

    async fn register_member(pool: &PgPool, github_user: GitHubUser) -> Result<Member, String> {
        let now = chrono::Utc::now().with_timezone(&Kolkata);

        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (name, email, github_user, role, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING member_id, roll_no, name, email, sex, year, hostel, mac_address, discord_id,
             group_id, track, github_user, role, created_at, updated_at",
        )
        .bind(github_user.name)
        .bind(github_user.email)
        .bind(github_user.github_username)
        .bind(Role::Member)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to register member: {}", e))?;

        Ok(member)
    }
}
