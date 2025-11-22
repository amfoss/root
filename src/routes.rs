use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Extension, Query as AxumQuery, State},
    http::{header, StatusCode},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::auth::auth_service::AuthService;
use crate::auth::middleware::auth_middleware;
use crate::auth::oauth::GitHubOAuthService;
use crate::auth::session::SessionService;
use crate::auth::AuthContext;
use crate::graphql::{Mutation, Query};
use crate::Config;

#[derive(Clone)]
struct AppState {
    schema: Schema<Query, Mutation, EmptySubscription>,
    pool: Arc<PgPool>,
    config: Config,
}

async fn graphql_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state
        .schema
        .execute(req.into_inner().data(auth_context))
        .await
        .into()
}

pub fn setup_router(
    schema: Schema<Query, Mutation, EmptySubscription>,
    cors: CorsLayer,
    config: Config,
    pool: Arc<PgPool>,
) -> Router {
    let pool_for_middleware = pool.clone();
    let app_state = AppState {
        schema,
        pool,
        config: config.clone(),
    };

    let router = Router::new()
        .route("/", post(graphql_handler))
        .route("/auth/github", get(github_oauth_init))
        .route("/auth/github/callback", get(github_oauth_callback))
        .route("/graphiql", get(graphiql).post(graphql_handler));

    router
        .layer(middleware::from_fn(move |req, next| {
            auth_middleware(pool_for_middleware.clone(), req, next)
        }))
        .layer(cors)
        .with_state(app_state)
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphiql")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

// OAuth handlers

/// Initiates GitHub OAuth flow
async fn github_oauth_init() -> Result<Redirect, String> {
    let oauth_service =
        GitHubOAuthService::new().map_err(|e| format!("Failed to initialize OAuth: {}", e))?;

    let (auth_url, _csrf_token) = oauth_service.get_authorization_url();

    Ok(Redirect::temporary(&auth_url))
}

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    // In a production system, this state variable should be populated
    // and verified using server side cookies. For now, we'll ignore it.
    #[allow(dead_code)]
    state: Option<String>,
}

/// GitHub OAuth callback handler - completes authentication and sets session cookie
async fn github_oauth_callback(
    State(state): State<AppState>,
    AxumQuery(query): AxumQuery<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let member = AuthService::handle_github_callback(state.pool.as_ref(), query.code)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("OAuth failed: {}", e)))?;

    let session_token = SessionService::create_session(state.pool.as_ref(), member.member_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create session: {}", e),
            )
        })?;

    let cookie = Cookie::build(("session_token", session_token))
        .path("/")
        .http_only(true)
        .secure(state.config.env != "development")
        .domain(state.config.hostname)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .build();

    // Redirect to frontend with cookie
    Ok((
        [(header::SET_COOKIE, cookie.to_string())],
        Redirect::to(&state.config.frontend_url),
    ))
}
