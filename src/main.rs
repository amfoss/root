use crate::graphql::mutations::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::routes::graphiql;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{routing::get, Router};
use chrono::Local;
use root::attendance::scheduled_task::scheduled_task;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::task;
use tokio::time::{sleep_until, Instant};
use tower_http::cors::{Any, CorsLayer};

mod db;
mod graphql;
mod leaderboard;
mod routes;

#[derive(Clone)]
struct MyState {
    pool: Arc<PgPool>,
    secret_key: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // TODO: Explain?
    env::set_var("PGOPTIONS", "-c ignore_version=true");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

    let pool = Arc::new(pool);
    let secret_key = secrets.get("ROOT_SECRET").expect("ROOT_SECRET not found");
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .data(secret_key.clone()) //
        .finish();

    let state = MyState {
        pool: pool.clone(),
        secret_key: secret_key.clone(),
    };

    // TODO: Restrict to amD and Home
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let router = Router::new()
        .route(
            "/",
            get(graphiql).post_service(GraphQL::new(schema.clone())),
        )
        .with_state(state)
        .layer(cors);

    task::spawn(async move {
        schedule_task_at_midnight(pool.clone()).await;
    });

    Ok(router.into())
}

// Sleep till midnight, then execute the task, repeat.
async fn schedule_task_at_midnight(pool: Arc<PgPool>) {
    loop {
        let now = Local::now();
        let next_midnight = (now + chrono::Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let duration_until_midnight = next_midnight.signed_duration_since(now.naive_local());
        let sleep_duration = tokio::time::Duration::from_secs(duration_until_midnight.num_seconds() as u64);

        sleep_until(Instant::now() + sleep_duration).await;
        scheduled_task(pool.clone()).await;
        // TODO: Use tracing
        print!("done");
    }
}
