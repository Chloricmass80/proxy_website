use std::{process::exit, fs};

use sqlx::{SqlitePool, Sqlite};
use tokio::net;
use axum::{Router, extract::{State}, response::Redirect, routing::get};

#[derive(Clone)]
struct AppState {
    pool : sqlx::Pool<Sqlite>
}

// THIS FUNCTION HANDLES WHAT ACTUALLY HAPPENS WHEN SOMEONE CONNECTS
async fn handle_connect(State(state): State<AppState>) -> Redirect {
    let _ = sqlx::query("UPDATE site_stats SET hits = hits + 1 WHERE id = 1")
        .execute(&state.pool)
        .await
        .expect("Unable to increment database hit count.");

    if let Ok(record) = sqlx::query_as::<_, (i64,)>("SELECT hits FROM site_stats WHERE id = 1")
        .fetch_one(&state.pool)
        .await 
    {
        println!("Persistent Hit Count: {}", record.0);
    }

    Redirect::to("https://en.wikipedia.org/wiki/Leonhard_Euler") // Redirect
}

fn create_db_file() -> Result<(), Box<dyn std::error::Error>> {
    if fs::exists("visitors.db")? {
        return Ok(())
    }

    fs::File::create_new("visitors.db")?;
    Ok(())
}

async fn create_app() -> Router {
    create_db_file()
    .expect("Unable to create visitors.db file.");

    let pool = SqlitePool::connect("sqlite:visitors.db").await
        .expect("Unable to initalize database!");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS site_stats (
            id INTEGER PRIMARY KEY,
            hits INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Unable to intalize tables.");

    sqlx::query("INSERT OR IGNORE INTO site_stats (id, hits) VALUES (1, 0)")
        .execute(&pool)
        .await
        .expect("Unable to intalize table values.");


    let state = AppState {
        pool,
    };

    Router::new()
        .route("/", get(handle_connect)).with_state(state)
}

#[tokio::main]
async fn main() {
    let app = create_app().await;

    let listener = match net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(v) => v,
        Err(e) => {
            println!("Could not start on port: {e}");
            exit(1);
        }
    };

    println!("Server running on http://localhost:3000");

    match axum::serve(listener, app).await {
        Ok(_) => {},
        Err(e) => {
            println!("Could not serve: {e}");
            exit(1)
        }
    }
}
