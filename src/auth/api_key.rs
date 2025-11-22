use crate::models::auth::{ApiKey, Role};
use crate::models::member::Member;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono_tz::Asia::Kolkata;
use rand::Rng;
use sqlx::PgPool;

const API_KEY_LENGTH: usize = 48;
const API_KEY_PREFIX: &str = "root_";

pub struct ApiKeyService;

impl ApiKeyService {
    fn generate_api_key() -> String {
        let mut rng = rand::thread_rng();
        let key: String = (0..API_KEY_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=25 => (b'A' + idx) as char,
                    26..=51 => (b'a' + (idx - 26)) as char,
                    _ => (b'0' + (idx - 52)) as char,
                }
            })
            .collect();
        format!("{}{}", API_KEY_PREFIX, key)
    }

    pub async fn create_api_key(
        pool: &PgPool,
        name: String,
        created_by: i32,
    ) -> Result<String, String> {
        let api_key = Self::generate_api_key();
        let key_hash =
            hash(&api_key, DEFAULT_COST).map_err(|e| format!("Failed to hash API key: {}", e))?;

        let _ = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO ApiKeys (name, key_hash, created_by)
            VALUES ($1, $2, $3)
            RETURNING
                api_key_id,
                name,
                key_hash,
                created_by,
                created_at,
                last_used_at
            "#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(created_by)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to create API key: {}", e))?;

        Ok(api_key)
    }

    pub async fn validate_api_key(pool: &PgPool, api_key: &str) -> Result<Option<Member>, String> {
        if !api_key.starts_with(API_KEY_PREFIX) {
            return Ok(None);
        }

        // Fetch all API keys (we need to bcrypt verify each one)
        let api_keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT
                api_key_id,
                name,
                key_hash,
                created_by,
                created_at,
                last_used_at
            FROM ApiKeys
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch API keys: {}", e))?;

        for key in api_keys {
            if verify(api_key, &key.key_hash).unwrap_or(false) {
                let _ = Self::update_last_used(pool, key.api_key_id).await;

                // Create a synthetic Member for the bot
                let bot_member = Member {
                    member_id: -(key.api_key_id), // Negative ID to distinguish from real members
                    roll_no: None,
                    name: key.name.clone(),
                    email: format!("bot-{}@internal.amfoss.in", key.api_key_id),
                    sex: None,
                    year: None,
                    hostel: None,
                    mac_address: None,
                    discord_id: None,
                    group_id: None,
                    track: None,
                    github_user: None,
                    role: Role::Bot,
                    created_at: key.created_at,
                    updated_at: key.created_at,
                };

                return Ok(Some(bot_member));
            }
        }

        Ok(None)
    }

    async fn update_last_used(pool: &PgPool, api_key_id: i32) -> Result<(), String> {
        let now = chrono::Utc::now().with_timezone(&Kolkata);

        sqlx::query(
            r#"
            UPDATE ApiKeys
            SET last_used_at = $1
            WHERE api_key_id = $2
            "#,
        )
        .bind(now)
        .bind(api_key_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update last_used_at: {}", e))?;

        Ok(())
    }

    pub async fn delete_api_key(pool: &PgPool, api_key_id: i32) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM ApiKeys
            WHERE api_key_id = $1
            "#,
        )
        .bind(api_key_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete API key: {}", e))?;

        Ok(())
    }
}
