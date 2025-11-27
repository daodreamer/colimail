// P3 Performance Benchmark: Database Queries
//
// This benchmark measures database query performance
// Target: INBOX list query < 50ms, batch inserts < 3 seconds for 100 emails

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    // Create in-memory database for benchmarking
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create schema
    sqlx::query(
        "CREATE TABLE accounts (
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
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE emails (
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
    .execute(&pool)
    .await
    .unwrap();

    // Create indexes
    sqlx::query("CREATE INDEX idx_emails_account_folder ON emails(account_id, folder_name)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE INDEX idx_emails_timestamp ON emails(timestamp DESC)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE INDEX idx_emails_seen ON emails(seen)")
        .execute(&pool)
        .await
        .unwrap();

    // Insert test account
    sqlx::query(
        "INSERT INTO accounts (id, email, imap_server, imap_port, smtp_server, smtp_port)
         VALUES (1, 'test@example.com', 'imap.example.com', 993, 'smtp.example.com', 587)",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn insert_test_emails(pool: &SqlitePool, count: usize) {
    for i in 0..count {
        let _ = sqlx::query(
            "INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr,
                                 date, timestamp, body, seen, flagged, synced_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'), ?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("INBOX")
        .bind(i as i32 + 1)
        .bind(format!("Test Subject {}", i))
        .bind(format!("sender{}@example.com", i))
        .bind("test@example.com")
        .bind(1704067200 + i as i64 * 3600) // Incrementing timestamp
        .bind(format!("<html><body>Email body {}</body></html>", i))
        .bind(i % 3 == 0) // Every 3rd email is read
        .bind(i % 10 == 0) // Every 10th email is flagged
        .bind(1704067200)
        .execute(pool)
        .await;
    }
}

fn benchmark_query_inbox_list(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("query_inbox_large_volumes");
    group.sample_size(10); // Reduce iterations for large datasets

    // Test with 1 million emails
    let pool_1m = rt.block_on(async {
        let pool = setup_test_db().await;
        println!("Inserting 1M test emails...");
        insert_test_emails(&pool, 1_000_000).await;
        println!("✅ 1M emails inserted");
        pool
    });

    group.bench_function("query_from_1M_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: Vec<(i64, String, String, i64)> = sqlx::query_as(
                "SELECT id, subject, from_addr, timestamp FROM emails
                 WHERE account_id = ? AND folder_name = ?
                 ORDER BY timestamp DESC
                 LIMIT 50",
            )
            .bind(1)
            .bind("INBOX")
            .fetch_all(black_box(&pool_1m))
            .await
            .unwrap();

            black_box(result);
        })
    });

    // Test with 10 million emails
    let pool_10m = rt.block_on(async {
        let pool = setup_test_db().await;
        println!("Inserting 10M test emails...");
        insert_test_emails(&pool, 10_000_000).await;
        println!("✅ 10M emails inserted");
        pool
    });

    group.bench_function("query_from_10M_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: Vec<(i64, String, String, i64)> = sqlx::query_as(
                "SELECT id, subject, from_addr, timestamp FROM emails
                 WHERE account_id = ? AND folder_name = ?
                 ORDER BY timestamp DESC
                 LIMIT 50",
            )
            .bind(1)
            .bind("INBOX")
            .fetch_all(black_box(&pool_10m))
            .await
            .unwrap();

            black_box(result);
        })
    });

    group.finish();
}

fn benchmark_query_unread_emails(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = rt.block_on(async {
        let pool = setup_test_db().await;
        insert_test_emails(&pool, 1000).await;
        pool
    });

    c.bench_function("query_unread_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: Vec<(i64,)> = sqlx::query_as(
                "SELECT id FROM emails
                 WHERE account_id = ? AND folder_name = ? AND seen = 0
                 ORDER BY timestamp DESC",
            )
            .bind(1)
            .bind("INBOX")
            .fetch_all(black_box(&pool))
            .await
            .unwrap();

            black_box(result);
        })
    });
}

fn benchmark_query_flagged_emails(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = rt.block_on(async {
        let pool = setup_test_db().await;
        insert_test_emails(&pool, 1000).await;
        pool
    });

    c.bench_function("query_flagged_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: Vec<(i64,)> = sqlx::query_as(
                "SELECT id FROM emails
                 WHERE account_id = ? AND flagged = 1
                 ORDER BY timestamp DESC",
            )
            .bind(1)
            .fetch_all(black_box(&pool))
            .await
            .unwrap();

            black_box(result);
        })
    });
}

fn benchmark_batch_insert(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_insert_emails");
    group.sample_size(10); // Reduce sample size for large inserts

    // Test various batch sizes including 1M and 10M
    for size in [100, 1_000, 10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let pool = setup_test_db().await;
                println!("Inserting {} emails...", size);
                insert_test_emails(black_box(&pool), size).await;
            })
        });
    }

    group.finish();
}

fn benchmark_search_by_subject(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = rt.block_on(async {
        let pool = setup_test_db().await;
        insert_test_emails(&pool, 1000).await;
        pool
    });

    c.bench_function("search_emails_by_subject", |b| {
        b.to_async(&rt).iter(|| async {
            let result: Vec<(i64,)> = sqlx::query_as(
                "SELECT id FROM emails
                 WHERE account_id = ? AND subject LIKE ?
                 ORDER BY timestamp DESC",
            )
            .bind(1)
            .bind("%Test%")
            .fetch_all(black_box(&pool))
            .await
            .unwrap();

            black_box(result);
        })
    });
}

fn benchmark_update_email_flags(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = rt.block_on(async {
        let pool = setup_test_db().await;
        insert_test_emails(&pool, 1000).await;
        pool
    });

    c.bench_function("update_email_seen_flag", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = sqlx::query(
                "UPDATE emails SET seen = 1
                 WHERE account_id = ? AND folder_name = ? AND uid = ?",
            )
            .bind(1)
            .bind("INBOX")
            .bind(1)
            .execute(black_box(&pool))
            .await;
        })
    });
}

fn benchmark_count_queries(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = rt.block_on(async {
        let pool = setup_test_db().await;
        insert_test_emails(&pool, 1000).await;
        pool
    });

    let mut group = c.benchmark_group("count_queries");

    group.bench_function("count_total_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM emails
                 WHERE account_id = ? AND folder_name = ?",
            )
            .bind(1)
            .bind("INBOX")
            .fetch_one(black_box(&pool))
            .await
            .unwrap();

            black_box(result);
        })
    });

    group.bench_function("count_unread_emails", |b| {
        b.to_async(&rt).iter(|| async {
            let result: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM emails
                 WHERE account_id = ? AND folder_name = ? AND seen = 0",
            )
            .bind(1)
            .bind("INBOX")
            .fetch_one(black_box(&pool))
            .await
            .unwrap();

            black_box(result);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_query_inbox_list,
    benchmark_query_unread_emails,
    benchmark_query_flagged_emails,
    benchmark_batch_insert,
    benchmark_search_by_subject,
    benchmark_update_email_flags,
    benchmark_count_queries
);

criterion_main!(benches);
