use crate::db;
use crate::models::{DraftListItem, DraftType};
use sqlx::Row;
use tauri::command;

/// Save draft to local database
#[command]
#[allow(clippy::too_many_arguments)]
pub async fn save_draft(
    account_id: i32,
    to_addr: String,
    cc_addr: String,
    subject: String,
    body: String,
    attachments: String,
    draft_type: DraftType,
    draft_id: Option<i64>,
) -> Result<i64, String> {
    let pool = db::pool();
    let now = chrono::Utc::now().timestamp();

    if let Some(id) = draft_id {
        // Update existing draft
        sqlx::query(
            "UPDATE drafts SET to_addr = ?, cc_addr = ?, subject = ?, body = ?,
             attachments = ?, draft_type = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&to_addr)
        .bind(&cc_addr)
        .bind(&subject)
        .bind(&body)
        .bind(&attachments)
        .bind(format!("{:?}", draft_type).to_lowercase())
        .bind(now)
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update draft: {}", e))?;

        Ok(id)
    } else {
        // Create new draft
        let result = sqlx::query(
            "INSERT INTO drafts (account_id, to_addr, cc_addr, subject, body,
             attachments, draft_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(&to_addr)
        .bind(&cc_addr)
        .bind(&subject)
        .bind(&body)
        .bind(&attachments)
        .bind(format!("{:?}", draft_type).to_lowercase())
        .bind(now)
        .bind(now)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to create draft: {}", e))?;

        Ok(result.last_insert_rowid())
    }
}

/// Load draft from local database
#[command]
pub async fn load_draft(
    draft_id: i64,
) -> Result<(String, String, String, String, String, String), String> {
    let pool = db::pool();

    let row = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT to_addr, cc_addr, subject, body, attachments, draft_type FROM drafts WHERE id = ?",
    )
    .bind(draft_id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load draft: {}", e))?;

    Ok(row)
}

/// List drafts from local database
#[command]
pub async fn list_drafts(account_id: i32) -> Result<Vec<DraftListItem>, String> {
    let pool = db::pool();

    let rows = sqlx::query(
        "SELECT id, account_id, to_addr, cc_addr, subject, draft_type, created_at, updated_at
         FROM drafts WHERE account_id = ? ORDER BY updated_at DESC",
    )
    .bind(account_id)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to list drafts: {}", e))?;

    let mut drafts = Vec::new();
    for row in rows {
        let draft_type_str: String = row
            .try_get("draft_type")
            .map_err(|e| format!("Failed to get draft_type: {}", e))?;
        let draft_type = serde_json::from_str::<DraftType>(&format!("\"{}\"", draft_type_str))
            .map_err(|e| format!("Failed to parse draft_type: {}", e))?;

        drafts.push(DraftListItem {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            account_id: row
                .try_get("account_id")
                .map_err(|e| format!("Failed to get account_id: {}", e))?,
            to_addr: row
                .try_get("to_addr")
                .map_err(|e| format!("Failed to get to_addr: {}", e))?,
            cc_addr: row
                .try_get("cc_addr")
                .map_err(|e| format!("Failed to get cc_addr: {}", e))?,
            subject: row
                .try_get("subject")
                .map_err(|e| format!("Failed to get subject: {}", e))?,
            draft_type,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        });
    }

    Ok(drafts)
}

/// Delete draft from local database
#[command]
pub async fn delete_draft(draft_id: i64) -> Result<(), String> {
    let pool = db::pool();

    sqlx::query("DELETE FROM drafts WHERE id = ?")
        .bind(draft_id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to delete draft: {}", e))?;

    Ok(())
}
