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

/// Daily task contains the function that is executed daily at midnight, using the thread spawned in main().
pub mod daily_task;
/// This module handles all logic for queries and mutations, based on the [`crate::models`]. Each sub-module maps to one table in the DB.
pub mod graphql;
/// These models not only help SQLx map it to the relational DB, but is also used by async_graphql to define its resolvers for queries and mutations.
pub mod models;
/// Since we really only need one route for a GraphQL server, this just holds a function returning the GraphiQL playground. Probably can clean this up later.
pub mod routes;

/// Handles all over environment variables in one place.
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
            // RUST_ENV is used to check if it's in production to avoid unnecessary logging and exposing the
            // graphiql interface. Make sure to set it to "production" before deployment.
            env: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
            // ROOT_SECRET is used to cryptographically verify the origin of attendance updation requests.
            secret_key: std::env::var("ROOT_SECRET").expect("ROOT_SECRET must be set."),
            // DATABASE_URL provides the connection string for the PostgreSQL database.
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            // ROOT_PORT is used to determine the port that root binds to
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

/// Abstraction over initializing the global subscriber for tracing depending on whether it's in production or dev.
fn setup_tracing(env: &str) {
    let kolkata_offset = UtcOffset::from_hms(5, 30, 0).expect("Hardcoded offset must be correct");
    let timer = fmt::time::OffsetTime::new(
        kolkata_offset,
        time::format_description::well_known::Rfc2822,
    );
    if env == "production" {
        tracing_subscriber::registry()
            // In production, no need to write to stdout, write directly to file.
            .with(
                fmt::layer()
                    .event_format(fmt::format().with_timer(timer.clone()))
                    .pretty()
                    .with_ansi(false) // ANSI encodings make it pretty but unreadable in the raw file.
                    .with_writer(std::fs::File::create("root.log").unwrap()),
            )
            // Allow only [`info`] and above events.
            .with(EnvFilter::new("info"))
            .init();
        info!("Running in production mode.")
    } else {
        tracing_subscriber::registry()
            // Write to both stdout and file in development.
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
            // Allow all events.
            .with(EnvFilter::new("trace"))
            .init();
        info!("Running in development mode.");
    }
}

/// Abstraction over setting up the database pool.
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

/// Abstraction over setting up the GraphQL schema from [`Query`] and [`Mutation`], and adding a reference to [`pool`] and [`secret_key`].
fn build_graphql_schema(
    pool: Arc<PgPool>,
    secret_key: String,
) -> async_graphql::Schema<Query, Mutation, EmptySubscription> {
    async_graphql::Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(pool)
        .data(secret_key)
        .finish()
}

/// Abstraction over making the CORSLayer.
fn setup_cors() -> CorsLayer {
    let origins: [HeaderValue; 2] = [
        "http://127.0.0.1:3000".parse().unwrap(),
        "https://home.amfoss.in".parse().unwrap(),
    ];

    CorsLayer::new()
        // Home should be the only website that accesses the API, bots and scripts do not trigger CORS AFAIK.
        // This lets us restrict who has access to what in the API on the Home frontend.
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
}
