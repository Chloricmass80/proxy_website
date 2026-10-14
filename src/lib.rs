use std::{fs};

use sqlx::{SqlitePool, Pool, Sqlite};

pub async fn increment_hit_count(pool: &Pool<Sqlite>) {
    let _ = sqlx::query("UPDATE site_stats SET hits = hits + 1 WHERE id = 1")
        .execute(pool)
        .await
        .expect("Unable to increment database hit count.");

    if let Ok(record) = sqlx::query_as::<_, (i64,)>("SELECT hits FROM site_stats WHERE id = 1")
        .fetch_one(pool)
        .await 
    {
        println!("Persistent Hit Count: {}", record.0);
    }
}

fn create_db_file() -> Result<(), Box<dyn std::error::Error>> {
    if fs::exists("visitors.db")? {
        return Ok(())
    }

    fs::File::create_new("visitors.db")?;
    Ok(())
}

pub async fn initalize_db() -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
    create_db_file()
    .expect("Unable to create visitors.db file.");

    let pool = SqlitePool::connect("sqlite:visitors.db").await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS site_stats (
            id INTEGER PRIMARY KEY,
            hits INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query("INSERT OR IGNORE INTO site_stats (id, hits) VALUES (1, 0)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn read_hits() -> Result<i64, Box<dyn std::error::Error>> {
    if !fs::exists("visitors.db")? {
        return Ok(0);
    }

    let pool = SqlitePool::connect("sqlite:visitors.db").await?;
    let record: (i64,) = sqlx::query_as("SELECT hits FROM site_stats WHERE id = 1")
        .fetch_one(&pool)
        .await?;

    Ok(record.0)
}