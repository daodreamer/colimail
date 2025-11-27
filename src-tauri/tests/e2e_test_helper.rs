// P3 E2E Test Helper - Database Fixture Creator
//
// This module creates test database fixtures for E2E testing

use sqlx::SqlitePool;

pub async fn create_test_fixture(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create database connection
    let db_url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&db_url).await?;

    // Create tables
    create_schema(&pool).await?;

    // Insert test data
    insert_test_accounts(&pool).await?;
    insert_test_folders(&pool).await?;
    insert_test_emails(&pool).await?;
    insert_test_attachments(&pool).await?;

    println!("✅ Test fixture created at: {}", db_path);

    Ok(pool)
}

async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create accounts table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            imap_server TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            smtp_server TEXT NOT NULL,
            smtp_port INTEGER NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'basic',
            display_name TEXT
        )",
    )
    .execute(pool)
    .await?;

    // Create folders table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            delimiter TEXT,
            flags TEXT,
            is_local INTEGER DEFAULT 0,
            UNIQUE(account_id, name),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await?;

    // Create emails table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS emails (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            folder_name TEXT NOT NULL,
            uid INTEGER NOT NULL,
            subject TEXT NOT NULL,
            from_addr TEXT NOT NULL,
            to_addr TEXT NOT NULL,
            cc_addr TEXT,
            date TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            body TEXT,
            raw_headers TEXT,
            has_attachments INTEGER DEFAULT 0,
            flags TEXT,
            seen INTEGER DEFAULT 0,
            flagged INTEGER DEFAULT 0,
            synced_at INTEGER NOT NULL,
            UNIQUE(account_id, folder_name, uid),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await?;

    // Create attachments table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY,
            email_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            content_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            data BLOB,
            FOREIGN KEY(email_id) REFERENCES emails(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_test_accounts(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO accounts (id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(1)
    .bind("test1@gmail.com")
    .bind("imap.gmail.com")
    .bind(993)
    .bind("smtp.gmail.com")
    .bind(587)
    .bind("oauth2")
    .bind("Test User 1")
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO accounts (id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(2)
    .bind("test2@outlook.com")
    .bind("outlook.office365.com")
    .bind(993)
    .bind("smtp.office365.com")
    .bind(587)
    .bind("oauth2")
    .bind("Test User 2")
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_test_folders(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Gmail folders
    let gmail_folders = vec![
        (1, 1, "INBOX", "Inbox"),
        (2, 1, "Sent", "Sent"),
        (3, 1, "Drafts", "Drafts"),
        (4, 1, "Trash", "Trash"),
    ];

    for (id, account_id, name, display_name) in gmail_folders {
        sqlx::query(
            "INSERT INTO folders (id, account_id, name, display_name, delimiter, flags, is_local)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .bind(display_name)
        .bind("/")
        .bind("")
        .bind(0)
        .execute(pool)
        .await?;
    }

    // Outlook folders
    let outlook_folders = vec![
        (5, 2, "INBOX", "Inbox"),
        (6, 2, "Sent Items", "Sent Items"),
        (7, 2, "Drafts", "Drafts"),
    ];

    for (id, account_id, name, display_name) in outlook_folders {
        sqlx::query(
            "INSERT INTO folders (id, account_id, name, display_name, delimiter, flags, is_local)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .bind(display_name)
        .bind("/")
        .bind("")
        .bind(0)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn insert_test_emails(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Insert sample emails (simplified version with 10 emails for testing)
    let emails = vec![
        (
            1,
            "INBOX",
            1,
            "Welcome to Colimail!",
            "welcome@colimail.com",
            "test1@gmail.com",
            1,
            0,
            1736935200,
        ),
        (
            1,
            "INBOX",
            2,
            "Project Update - Q1 2025",
            "manager@company.com",
            "test1@gmail.com",
            1,
            1,
            1737020400,
        ),
        (
            1,
            "INBOX",
            3,
            "Meeting Reminder",
            "calendar@company.com",
            "test1@gmail.com",
            1,
            0,
            1737036000,
        ),
        (
            1,
            "INBOX",
            4,
            "Newsletter: Tech Weekly",
            "newsletter@tech.com",
            "test1@gmail.com",
            0,
            0,
            1737097200,
        ),
        (
            1,
            "INBOX",
            5,
            "Invoice #12345",
            "billing@service.com",
            "test1@gmail.com",
            0,
            1,
            1737108000,
        ),
        (
            1,
            "Sent",
            6,
            "Re: Project proposal",
            "test1@gmail.com",
            "client@client.com",
            1,
            0,
            1736953200,
        ),
        (
            1,
            "Sent",
            7,
            "Meeting notes",
            "test1@gmail.com",
            "team@company.com",
            1,
            0,
            1737046800,
        ),
        (
            1,
            "Drafts",
            8,
            "Draft: Quarterly report",
            "test1@gmail.com",
            "boss@company.com",
            0,
            0,
            1738231200,
        ),
        (
            2,
            "INBOX",
            9,
            "Welcome to Outlook",
            "welcome@outlook.com",
            "test2@outlook.com",
            1,
            0,
            1736931600,
        ),
        (
            2,
            "INBOX",
            10,
            "Team Meeting Agenda",
            "manager@company.com",
            "test2@outlook.com",
            0,
            1,
            1737021600,
        ),
    ];

    for (account_id, folder, uid, subject, from_addr, to_addr, seen, flagged, timestamp) in emails {
        let body = format!(
            "<html><body><p>Test email body for: {}</p></body></html>",
            subject
        );
        sqlx::query(
            "INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr,
                                 date, timestamp, body, seen, flagged, synced_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime(?, 'unixepoch'), ?, ?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(folder)
        .bind(uid)
        .bind(subject)
        .bind(from_addr)
        .bind(to_addr)
        .bind(timestamp)
        .bind(timestamp)
        .bind(body)
        .bind(seen)
        .bind(flagged)
        .bind(timestamp)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn insert_test_attachments(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Insert sample attachments for emails with has_attachments=1
    sqlx::query(
        "INSERT INTO attachments (email_id, filename, content_type, size)
         VALUES (?, ?, ?, ?)",
    )
    .bind(2) // Project Update email
    .bind("project_update.pdf")
    .bind("application/pdf")
    .bind(245680)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_create_fixture() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let pool = create_test_fixture(db_path).await.unwrap();

        // Verify accounts
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "Should have 2 test accounts");

        // Verify folders
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM folders")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count.0 >= 4, "Should have at least 4 folders");

        // Verify emails
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count.0 >= 10, "Should have at least 10 test emails");

        println!("✅ Fixture validation passed");
    }
}
