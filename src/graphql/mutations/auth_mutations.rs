use crate::auth::api_key::ApiKeyService;
use crate::auth::guards::AdminGuard;
use crate::auth::AuthContext;
use crate::models::auth::ApiKeyResponse;
use async_graphql::{Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct AuthMutations;

#[Object]
impl AuthMutations {
    /// Create a new bot with API key (Admin only)
    #[graphql(name = "createBot", guard = "AdminGuard")]
    async fn create_bot(&self, ctx: &Context<'_>, name: String) -> Result<ApiKeyResponse> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");
        let auth = ctx
            .data::<AuthContext>()
            .expect("AuthContext must be in context.");

        let admin_member = auth
            .user
            .as_ref()
            .ok_or("Admin member not found in context")?;

        // Create API key
        let api_key = ApiKeyService::create_api_key(pool.as_ref(), name, admin_member.member_id)
            .await
            .map_err(|e| format!("Failed to create bot: {}", e))?;

        Ok(ApiKeyResponse { api_key })
    }
}
