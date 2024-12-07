use std::sync::Arc;
use sqlx::PgPool;

pub async fn update_leaderboard(pool: Arc<PgPool>) -> Result<(), Box<dyn std::error::Error>> {
    
    let leetcode_stats: Result<Vec<(i32, i32, i32, i32, i32, i32, i32, i32)>, _> =
        sqlx::query_as::<_, (i32, i32, i32, i32, i32, i32, i32, i32)>(
            "SELECT id, member_id, problems_solved, easy_solved, medium_solved, hard_solved, contests_participated, best_rank 
             FROM leetcode_stats",
        )
        .fetch_all(pool.as_ref())
        .await;

    let leetcode_stats = match leetcode_stats {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Failed to fetch LeetCode stats: {:?}", e);
            return Err(Box::new(e));
        }
    };


    let codeforces_stats: Result<Vec<(i32, i32, i32, i32, i32)>, _> =
        sqlx::query_as::<_, (i32, i32, i32, i32, i32)>(
            "SELECT id, member_id, codeforces_rating, max_rating, contests_participated 
             FROM codeforces_stats",
        )
        .fetch_all(pool.as_ref())
        .await;

    let codeforces_stats = match codeforces_stats {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Failed to fetch Codeforces stats: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // Create a lookup table for Codeforces stats
    let cf_lookup: std::collections::HashMap<i32, (i32, i32, i32)> = codeforces_stats
        .iter()
        .map(
            |(_, member_id, codeforces_rating, max_rating, contests_participated)| {
                (
                    *member_id,
                    (*codeforces_rating, *max_rating, *contests_participated),
                )
            },
        )
        .collect();

   
    for (
        _,
        member_id,
        _,
        easy_solved,
        medium_solved,
        hard_solved,
        contests_participated,
        best_rank,
    ) in &leetcode_stats
    {
        //todo: the algorith is not correct, might have to find a good one
        let leetcode_score = (5 * easy_solved)
            + (10 * medium_solved)
            + (20 * hard_solved)
            + (2 * contests_participated)
            + (100 - best_rank / 10).max(0);

        let codeforces_score = cf_lookup
            .get(member_id)
            .map_or(0, |(rating, max_rating, contests)| {
                (rating / 10) + (max_rating / 20) + (5 * contests)
            });

        let unified_score = leetcode_score + codeforces_score;

        let result = sqlx::query(
            "INSERT INTO leaderboard (member_id, leetcode_score, codeforces_score, unified_score, last_updated)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (member_id) DO UPDATE SET
                 leetcode_score = EXCLUDED.leetcode_score,
                 codeforces_score = EXCLUDED.codeforces_score,
                 unified_score = EXCLUDED.unified_score,
                 last_updated = NOW()",
        )
        .bind(member_id)
        .bind(leetcode_score)
        .bind(codeforces_score)
        .bind(unified_score)
        .execute(pool.as_ref())
        .await;

        if let Err(e) = result {
            eprintln!(
                "Failed to update leaderboard for member ID: {}: {:?}",
                member_id, e
            );
        }
    }

    // Process remaining Codeforces-only members
    for (_, member_id, codeforces_rating, max_rating, contests_participated) in &codeforces_stats {
        if leetcode_stats
            .iter()
            .any(|(_, lc_member_id, _, _, _, _, _, _)| lc_member_id == member_id)
        {
            continue;
        }

        let codeforces_score =
            (codeforces_rating / 10) + (max_rating / 20) + (5 * contests_participated);

        // Default LeetCode score is 0
        let leetcode_score = 0;
        let unified_score = codeforces_score;

        let result = sqlx::query(
            "INSERT INTO leaderboard (member_id, leetcode_score, codeforces_score, unified_score, last_updated)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (member_id) DO UPDATE SET
                 leetcode_score = EXCLUDED.leetcode_score,
                 codeforces_score = EXCLUDED.codeforces_score,
                 unified_score = EXCLUDED.unified_score,
                 last_updated = NOW()",
        )
        .bind(member_id)
        .bind(leetcode_score)
        .bind(codeforces_score)
        .bind(unified_score)
        .execute(pool.as_ref())
        .await;

        if let Err(e) = result {
            eprintln!(
                "Failed to update leaderboard for member ID: {}: {:?}",
                member_id, e
            );
        }
    }

    Ok(())
}
