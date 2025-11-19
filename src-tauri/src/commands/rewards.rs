use crate::db;
use crate::models::{RewardCacheItem, RewardStatus};
use tauri::command;

#[command]
pub async fn save_reward_cache(rewards: Vec<RewardCacheItem>) -> Result<(), String> {
    let pool = db::pool();

    for reward in rewards {
        let status_str = serde_json::to_string(&reward.status)
            .map_err(|e| e.to_string())?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            "INSERT OR REPLACE INTO reward_cache 
            (reward_id, email_hash, amount, sender, recipient, status, timestamp, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(reward.reward_id)
        .bind(reward.email_hash)
        .bind(reward.amount)
        .bind(reward.sender)
        .bind(reward.recipient)
        .bind(status_str)
        .bind(reward.timestamp)
        .bind(reward.updated_at)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
pub async fn get_reward_cache(email_hash: String) -> Result<Option<RewardCacheItem>, String> {
    let pool = db::pool();

    let row = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64)>(
        "SELECT reward_id, email_hash, amount, sender, recipient, status, timestamp, updated_at 
         FROM reward_cache WHERE email_hash = ?",
    )
    .bind(email_hash)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((
        reward_id,
        email_hash,
        amount,
        sender,
        recipient,
        status_str,
        timestamp,
        updated_at,
    )) = row
    {
        let status = match status_str.as_str() {
            "available" => RewardStatus::Available,
            "claimed" => RewardStatus::Claimed,
            "cancelled" => RewardStatus::Cancelled,
            _ => RewardStatus::Available, // Default fallback
        };

        Ok(Some(RewardCacheItem {
            reward_id,
            email_hash,
            amount,
            sender,
            recipient,
            status,
            timestamp,
            updated_at,
        }))
    } else {
        Ok(None)
    }
}

#[command]
pub async fn get_all_reward_caches() -> Result<Vec<RewardCacheItem>, String> {
    let pool = db::pool();

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64)>(
        "SELECT reward_id, email_hash, amount, sender, recipient, status, timestamp, updated_at 
         FROM reward_cache ORDER BY timestamp DESC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let rewards = rows
        .into_iter()
        .map(
            |(
                reward_id,
                email_hash,
                amount,
                sender,
                recipient,
                status_str,
                timestamp,
                updated_at,
            )| {
                let status = match status_str.as_str() {
                    "available" => RewardStatus::Available,
                    "claimed" => RewardStatus::Claimed,
                    "cancelled" => RewardStatus::Cancelled,
                    _ => RewardStatus::Available,
                };

                RewardCacheItem {
                    reward_id,
                    email_hash,
                    amount,
                    sender,
                    recipient,
                    status,
                    timestamp,
                    updated_at,
                }
            },
        )
        .collect();

    Ok(rewards)
}
