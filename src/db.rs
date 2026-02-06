use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
use std::fs::File;
use std::path::Path;

pub async fn init_db() -> Result<Pool<Sqlite>, Error> {
    if !Path::new("arkheion.db").exists() {
        File::create("arkheion.db").expect("Gagal membuat file database");
        println!(">>> Database file created: arkheion.db");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5) 
        .connect("sqlite://arkheion.db?mode=rwc")
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_key TEXT UNIQUE NOT NULL,
            api_key TEXT UNIQUE NOT NULL,
            tier TEXT DEFAULT 'free',
            credits INTEGER DEFAULT 1000,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_active DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS payment_tx (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            signature TEXT UNIQUE NOT NULL,
            amount_sol REAL NOT NULL,
            status TEXT DEFAULT 'pending',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id)
        )",
    )
    .execute(&pool)
    .await?;

    println!(">>> Database initialized successfully.");
    Ok(pool)
}
