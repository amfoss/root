use config::{Config, File, FileFormat};
use root::db::leaderboard::Leaderboard;
use root::db::member::Member;
use root::leaderboard::fetch_stats::{fetch_codeforces_stats, fetch_leetcode_stats};
use root::leaderboard::update_leaderboard::update_leaderboard;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

pub fn get_database_url() -> String {
    // Create a configuration instance to read Secrets.toml
    let settings = Config::builder()
        .add_source(File::new("Secrets", FileFormat::Toml))
        .build()
        .expect("Failed to load Secrets.toml");

    // Retrieve the `DATABASE_URL` from the file
    settings
        .get_string("DATABASE_URL")
        .expect("Missing 'DATABASE_URL' in Secrets.toml")
}

// Helper function to create a test database connection
async fn setup_test_db() -> PgPool {
    let database_url = get_database_url();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &PgPool) {
    // sqlx::query("DELETE FROM leaderboard")
    //     .execute(pool)
    //     .await
    //     .unwrap();
    // sqlx::query("DELETE FROM leetcode_stats")
    //     .execute(pool)
    //     .await
    //     .unwrap();
    // sqlx::query("DELETE FROM codeforces_stats")
    //     .execute(pool)
    //     .await
    //     .unwrap();
    // sqlx::query("DELETE FROM Member")
    //     .execute(pool)
    //     .await
    //     .unwrap();
}

//test
#[tokio::test]
async fn test_insert_members_and_update_stats() {
    let pool = setup_test_db().await;

    cleanup_test_data(&pool).await;

    // Define test members
    let members = vec![
        (
            "B21CS1234",
            "John Doe",
            "Hostel A",
            "john.doe@example.com",
            "Male",
            2021,
            "00:11:22:33:44:55",
            Some("john_discord"),
            "swayam-agrahari",
            "tourist",
        ),
        (
            "B21CS5678",
            "Jane Smith",
            "Hostel B",
            "jane.smith@example.com",
            "Female",
            2021,
            "66:77:88:99:AA:BB",
            Some("jane_discord"),
            "rihaan1810",
            "tourist",
        ),
    ];

    let mut inserted_members = Vec::new();

    // Insert members and store their IDs
    for member in &members {
        // Insert Member
        let member_result = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (rollno, name, hostel, email, sex, year, macaddress, discord_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 RETURNING *",
        )
        .bind(&member.0)
        .bind(&member.1)
        .bind(&member.2)
        .bind(&member.3)
        .bind(&member.4)
        .bind(member.5)
        .bind(&member.6)
        .bind(&member.7)
        .fetch_one(&pool)
        .await
        .expect("Failed to insert member");

        // Insert LeetCode stats
        let _leetcode_result = sqlx::query(
                "INSERT INTO leetcode_stats (member_id, leetcode_username,problems_solved,easy_solved,medium_solved,hard_solved,contests_participated,best_rank,total_contests)
                 VALUES ($1, $2, 0,0,0,0,0,0,0)",
            )
            .bind(member_result.id)
            .bind(&member.8)
            .execute(&pool)
            .await
            .expect("Failed to insert LeetCode stats");

        // Insert Codeforces stats
        let _codeforces_result = sqlx::query(
                "INSERT INTO codeforces_stats (member_id, codeforces_handle,codeforces_rating,max_rating,contests_participated)
                 VALUES ($1, $2, 0,0,0)",
            )
            .bind(member_result.id)
            .bind(&member.9)
            .execute(&pool)
            .await
            .expect("Failed to insert Codeforces stats");

        inserted_members.push(member_result.id);
    }

    // Test LeetCode stats fetching
    for (member_id, leetcode_username) in inserted_members.iter().zip(members.iter().map(|m| m.8)) {
        match fetch_leetcode_stats(Arc::new(pool.clone()), *member_id, leetcode_username).await {
            Ok(_) => println!(
                "Successfully fetched LeetCode stats for member ID: {}",
                member_id
            ),
            Err(e) => {
                println!("Error fetching LeetCode stats: {:?}", e);
                // Uncomment to fail test on fetch error
                // panic!("Failed to fetch LeetCode stats")
            }
        }
    }

    // Test Codeforces stats fetching
    for (member_id, codeforces_handle) in inserted_members.iter().zip(members.iter().map(|m| m.9)) {
        match fetch_codeforces_stats(Arc::new(pool.clone()), *member_id, codeforces_handle).await {
            Ok(_) => println!(
                "Successfully fetched Codeforces stats for member ID: {}",
                member_id
            ),
            Err(e) => {
                println!("Error fetching Codeforces stats: {:?}", e);
                // Uncomment to fail test on fetch error
                // panic!("Failed to fetch Codeforces stats")
            }
        }
    }

    // Test leaderboard update
    match update_leaderboard(Arc::new(pool.clone())).await {
        Ok(_) => println!("Successfully updated leaderboard"),
        Err(e) => panic!("Failed to update leaderboard: {:?}", e),
    }

    // Verify leaderboard entries
    let leaderboard_entries = sqlx::query_as::<_, Leaderboard>("SELECT * FROM leaderboard")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(
        leaderboard_entries.len(),
        2,
        "Should have 2 leaderboard entries"
    );

    // Assertions about leaderboard scores
    for entry in leaderboard_entries {
        assert!(entry.unified_score > 0, "Unified score should be positive");
        assert!(
            entry.leetcode_score.is_some(),
            "LeetCode score should be set"
        );
        assert!(
            entry.codeforces_score.is_some(),
            "Codeforces score should be set"
        );
    }

    // Clean up
    cleanup_test_data(&pool).await;
}

// Additional helper test to verify database connections and basic operations
#[tokio::test]
async fn test_database_connection() {
    let pool = setup_test_db().await;
    let database_url = get_database_url();

    // Print the URL to verify (optional, for debugging purposes)
    println!("Database URL: {}", database_url);

    // Basic database connectivity test
    let result = sqlx::query("SELECT 1").fetch_one(&pool).await;

    assert!(result.is_ok(), "Database connection and query should work");
}
