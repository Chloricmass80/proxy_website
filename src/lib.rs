use std::{env, fs, process::exit};

use sqlx::{SqlitePool, Pool, Sqlite};

pub async fn increment_hit_count(pool: &Pool<Sqlite>) {
    if let Err(e) = sqlx::query("UPDATE site_stats SET hits = hits + 1 WHERE id = 1")
        .execute(pool)
        .await{
            eprintln!("Unable to increment database hitcount: {e}")
        };

    match sqlx::query_as::<_, (i64,)>("SELECT hits FROM site_stats WHERE id = 1")
        .fetch_one(pool)
        .await {
        Ok(record) => {
            println!("Persistent Hit Count: {}", record.0);
        },
        Err(e) => {
            eprintln!("Unable to fetch Hit Count: {e}");
        }
    };
}

fn create_db_file() -> Result<(), Box<dyn std::error::Error>> {
    if fs::exists("visitors.db")? {
        return Ok(())
    }

    fs::File::create_new("visitors.db")?;
    Ok(())
}

pub async fn initalize_db() -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
    create_db_file()?;

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

pub fn load_env_variables() -> Result<(), dotenvy::Error> {
    match dotenvy::from_filename("config/local.env") {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.not_found() {
                let mut env_path = env::current_exe().map_or_else(|e| {
                    eprintln!("!FATAL! Could not find current executable's file path: {e}");
                    exit(1);
                }, |v| v);
                env_path.pop();
                env_path.push(".env");

                // try finding by ../.env
                if fs::exists(&env_path).is_ok_and(|x| x) {
                    dotenvy::from_filename(env_path)?;
                // try finding by ../local.env
                } else {
                    env_path.pop();
                    env_path.push("local.env");
                    dotenvy::from_filename(env_path)?;
                }

                Ok(())
            } else {
                Err(e)
            }
        }
    }
}