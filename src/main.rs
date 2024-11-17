use crate::graphql::mutations::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::routes::graphiql;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{routing::get, Router};
use chrono::{Local, NaiveTime};
use chrono_tz::Asia::Kolkata;
use db::member::Member;
use reqwest;
use serde_json::Value;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::time::Duration;
use std::{env, sync::Arc};
use tokio::task;
use tokio::time::{sleep_until, Instant};
use tower_http::cors::{Any, CorsLayer};

mod db;
mod graphql;
mod routes;

#[derive(Clone)]
struct MyState {
    pool: Arc<PgPool>,
    secret_key: String,
}

//Main method
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    env::set_var("PGOPTIONS", "-c ignore_version=true");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

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

    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow any origin
        .allow_methods(tower_http::cors::Any) // Allow any HTTP method
        .allow_headers(tower_http::cors::Any);

    let router = Router::new()
        .route(
            "/",
            get(graphiql).post_service(GraphQL::new(schema.clone())),
        )
        .with_state(state)
        .layer(cors);
    task::spawn(async move {
        schedule_task_at_midnight(pool.clone()).await; // Call the function after 10 seconds
    });

    Ok(router.into())
}

//Scheduled task for moving all members to Attendance table at midnight.
async fn scheduled_task(pool: Arc<PgPool>) {
    let members: Result<Vec<Member>, sqlx::Error> =
        sqlx::query_as::<_, Member>("SELECT * FROM Member")
            .fetch_all(pool.as_ref())
            .await;

    match members {
        Ok(members) => {
            let today = Local::now().with_timezone(&Kolkata);

            for member in members {
                let timein = NaiveTime::from_hms_opt(0, 0, 0);
                let timeout = NaiveTime::from_hms_opt(0, 0, 0); // Default time, can be modified as needed

                let attendance = sqlx::query(
                    "INSERT INTO Attendance (id, date, timein, timeout, is_present) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id, date) DO NOTHING RETURNING *"
                )
                .bind(member.id)
                .bind(today)
                .bind(timein)
                .bind(timeout)
                .bind(false)
                .execute(pool.as_ref())
                .await;

                match attendance {
                    Ok(_) => println!("Attendance record added for member ID: {}", member.id),
                    Err(e) => eprintln!(
                        "Failed to insert attendance for member ID: {}: {:?}",
                        member.id, e
                    ),
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch members: {:?}", e),
    }
    // Update CP ratings
    update_cp_ratings(pool.clone()).await;
    println!("Ratings updated successfully.");
}

//Ticker for calling the scheduled task
async fn schedule_task_at_midnight(pool: Arc<PgPool>) {
    loop {
        let now = Local::now();

        let tomorrow = now.date_naive().succ_opt().unwrap();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let next_midnight = tomorrow.and_time(midnight);

        let now_naive = now.naive_local();
        let duration_until_midnight = next_midnight.signed_duration_since(now_naive);
        let sleep_duration = Duration::from_secs(duration_until_midnight.num_seconds() as u64 + 60);

        sleep_until(Instant::now() + sleep_duration).await;
        scheduled_task(pool.clone()).await;
        print!("done");
    }
}
// Function to fetch codeforces ranking
async fn fetch_codeforces_rating(
    username: &str,
) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let url = format!("https://codeforces.com/api/user.rating?handle={}", username);
    let response = reqwest::get(&url).await?.text().await?;
    let data: Value = serde_json::from_str(&response)?;

    if data["status"] == "OK" {
        if let Some(results) = data["result"].as_array() {
            if let Some(last_contest) = results.last() {
                let new_rating = last_contest["newRating"].as_i64().unwrap_or_default() as i32;
                return Ok(Some(new_rating));
            }
        }
    }
    Ok(None)
}

// Function to fetch LeetCode ranking
async fn fetch_leetcode_ranking(username: &str) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://leetcode.com/graphql";
    let query = r#"
        query userPublicProfile($username: String!) {
            matchedUser(username: $username) {
                profile {
                    ranking
                }
            }
        }
    "#;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": {
                "username": username
            }
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let ranking = data["data"]["matchedUser"]["profile"]["ranking"]
        .as_i64()
        .map(|v| v as i32);

    Ok(ranking)
}

// Fetch and update CP ratings for all members
async fn update_cp_ratings(pool: Arc<PgPool>) {
    let members: Result<Vec<Member>, sqlx::Error> =
        sqlx::query_as::<_, Member>("SELECT * FROM Member")
            .fetch_all(pool.as_ref())
            .await;

    match members {
        Ok(members) => {
            for member in members {
                let rating = match member.cp_platform.as_str() {
                    "Codeforces" => fetch_codeforces_rating(&member.leaderboard_id)
                        .await
                        .ok()
                        .flatten(),
                    "LeetCode" => fetch_leetcode_ranking(&member.leaderboard_id)
                        .await
                        .ok()
                        .flatten(),
                    _ => None,
                };

                if let Some(rating) = rating {
                    let update_result = sqlx::query("UPDATE Member SET rating = $1 WHERE id = $2")
                        .bind(rating)
                        .bind(member.id)
                        .execute(pool.as_ref())
                        .await;

                    match update_result {
                        Ok(_) => println!("Updated rating for {}: {}", member.name, rating),
                        Err(e) => eprintln!("Failed to update rating for {}: {:?}", member.name, e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch members: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mocking the PgPool for testing purposes (if necessary)
    use sqlx::PgPool;

    #[tokio::test]
    // Update these variables with the actual values before running the test

    async fn test_fetch_codeforces_rating() {
        let codeforces_username = ""; // Add your Codeforces username here
        let result = fetch_codeforces_rating(codeforces_username).await;
        assert!(result.is_ok());
        let rating = result.unwrap();
        assert!(rating.is_some());
    }

    #[tokio::test]
    async fn test_fetch_leetcode_ranking() {
        let leetcode_username = ""; // Add your LeetCode username here
        let result = fetch_leetcode_ranking(leetcode_username).await;
        assert!(result.is_ok());
        let ranking = result.unwrap();
        assert!(ranking.is_some());
    }

    #[tokio::test]
    async fn test_scheduled_task() {
        let database_url = ""; // Add your database URL here
        let pool = Arc::new(PgPool::connect_lazy(database_url).unwrap());

        scheduled_task(pool).await;
    }

    // Test for update_cp_ratings
    #[tokio::test]
    async fn test_update_cp_ratings() {
        let database_url = ""; // Add your database URL here
        let pool = Arc::new(PgPool::connect(database_url).await.unwrap());
        update_cp_ratings(pool).await;
    }
}
