use async_graphql::EmptySubscription;
use axum::http::{HeaderValue, Method};
use sqlx::PgPool;
use std::sync::Arc;
use time::UtcOffset;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use daily_task::run_daily_task_at_midnight;
use graphql::{Mutation, Query};
use routes::setup_router;

pub mod daily_task;
pub mod graphql;
pub mod models;
pub mod routes;

/// Handles all over environment variables in one place.
// TODO: Replace with `Config.rs` crate.
struct Config {
    env: String,
    secret_key: String,
    database_url: String,
    port: String,
}

impl Config {
    fn from_env() -> Self {
        let _ = dotenv::dotenv();
        Self {
            env: std::env::var("ROOT_ENV").unwrap_or_else(|_| "development".to_string()),
            secret_key: std::env::var("ROOT_SECRET").expect("ROOT_SECRET must be set."),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            port: std::env::var("ROOT_PORT").expect("ROOT_PORT must be set."),
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    setup_tracing(&config.env);

    let pool = setup_database(&config.database_url).await;
    let schema = build_graphql_schema(pool.clone(), config.secret_key);

    tokio::task::spawn(async {
        run_daily_task_at_midnight(pool).await;
    });

    let cors = setup_cors();
    let router = setup_router(schema, cors, config.env == "development");

    info!("Starting Root...");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}

fn setup_tracing(env: &str) {
    let kolkata_offset = UtcOffset::from_hms(5, 30, 0).expect("Hardcoded offset must be correct");
    let timer = fmt::time::OffsetTime::new(
        kolkata_offset,
        time::format_description::well_known::Rfc2822,
    );
    if env == "production" {
        tracing_subscriber::registry()
            // Don't waste resources writing to unmonitored stdout in production
            .with(
                fmt::layer()
                    .event_format(fmt::format().with_timer(timer.clone()))
                    .pretty()
                    .with_ansi(false) // ANSI encodings are unreadable in the raw file.
                    .with_writer(std::fs::File::create("root.log").unwrap()),
            )
            .with(EnvFilter::new("info"))
            .init();
        info!("Running in production mode.")
    } else {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .event_format(fmt::format().with_timer(timer.clone()))
                    .pretty()
                    .with_writer(std::io::stdout),
            )
            .with(
                fmt::layer()
                    .event_format(fmt::format().with_timer(timer.clone()))
                    .pretty()
                    .with_ansi(false)
                    .with_writer(std::fs::File::create("root.log").unwrap()),
            )
            .with(EnvFilter::new("trace"))
            .init();
        info!("Running in development mode.");
    }
}

async fn setup_database(database_url: &str) -> Arc<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(2)
        .max_connections(3)
        .connect(database_url)
        .await
        .expect("Pool must be initialized properly.");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

    Arc::new(pool)
}

fn build_graphql_schema(
    pool: Arc<PgPool>,
    secret_key: String,
) -> async_graphql::Schema<Query, Mutation, EmptySubscription> {
    async_graphql::Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(pool)
        .data(secret_key)
        .finish()
}

fn setup_cors() -> CorsLayer {
    // TODO: Replace hardcoded strings
    let origins: [HeaderValue; 2] = [
        "http://127.0.0.1:3000".parse().unwrap(),
        "https://home.amfoss.in".parse().unwrap(),
    ];

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
}
