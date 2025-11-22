use crate::auth::api_key::ApiKeyService;
use crate::auth::session::SessionService;
use crate::auth::AuthContext;
use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn auth_middleware(
    pool: Arc<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let jar = CookieJar::from_headers(request.headers());

    let member = if let Some(cookie) = jar.get("session_token") {
        SessionService::validate_session(&pool, cookie.value())
            .await
            .ok()
            .flatten()
    } else if let Some(auth_value) = auth_header {
        let token = auth_value.strip_prefix("Bearer ").unwrap_or(auth_value);
        ApiKeyService::validate_api_key(&pool, token)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    // Inject auth context into request extensions
    request.extensions_mut().insert(AuthContext::new(member));

    Ok(next.run(request).await)
}
