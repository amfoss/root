use crate::models::member::Member;
use chrono::{Duration, Utc};
use chrono_tz::Asia::Kolkata;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

const SESSION_DURATION_DAYS: i64 = 30;
const TOKEN_LENGTH: usize = 64;

pub struct SessionService;

impl SessionService {
    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let token: String = (0..TOKEN_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=25 => (b'A' + idx) as char,
                    26..=51 => (b'a' + (idx - 26)) as char,
                    _ => (b'0' + (idx - 52)) as char,
                }
            })
            .collect();
        token
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn create_session(pool: &PgPool, member_id: i32) -> Result<String, String> {
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now().with_timezone(&Kolkata) + Duration::days(SESSION_DURATION_DAYS);

        sqlx::query(
            r#"
            INSERT INTO Sessions (member_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(member_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create session: {}", e))?;

        Ok(token)
    }

    pub async fn validate_session(pool: &PgPool, token: &str) -> Result<Option<Member>, String> {
        let token_hash = Self::hash_token(token);
        let now = chrono::Utc::now().with_timezone(&Kolkata);

        let result = sqlx::query_as::<_, Member>(
            r#"
            SELECT m.* FROM Member m
            INNER JOIN Sessions s ON m.member_id = s.member_id
            WHERE s.token_hash = $1 AND s.expires_at > $2
            "#,
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to validate session: {}", e))?;

        Ok(result)
    }

    pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<u64, String> {
        let now = chrono::Utc::now().with_timezone(&Kolkata);

        let result = sqlx::query(
            r#"
            DELETE FROM Sessions
            WHERE expires_at <= $1
            "#,
        )
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to cleanup sessions: {}", e))?;

        Ok(result.rows_affected())
    }
}
