use std::{process::exit, env};

use sqlx::{Pool, Sqlite};
use tokio::net;
use axum::{Router, extract::{State}, response::Redirect, routing::get};
use dotenvy;

use proxy_website::{self, initalize_db};

#[derive(Clone)]
struct AppState {
    pool : sqlx::Pool<Sqlite>,
    config : Config
}

#[derive(Debug, Clone)]
struct Config {
    redirect_uri: String,
    track_hits: bool,
}

impl Config {
    async fn new(args: Vec<String>) -> Self {
        dotenvy::from_filename("config/local.env").map_err(|e| {
            println!("Unable to load environment variables from config/local.env.\n
Please make sure it exists.\n
Error: {e}");
            exit(1);
        });

        if args.get(1).is_some() {
            let help = args[1].to_lowercase() == "-h".to_string() || args[1].to_lowercase() == "--help".to_string();
            if help {
                println!("proxy_website help:\n
This binary is meant to run on a server as a backend.
It tracks the number of hits it gets, then immediately redirects\n\n
To read hits, add an argument '-r' or '--hits'. To instead see the number of hits. This will not run the server.\n
Environment Variable setup:\n
Find environment variables in local.env.\n
To see what environment variables exist, visit example.env.\n
To set a bool environment variable to false, simply remove it's decleration.\n
To set a bool environment variable to true, add it's decleration to equal any value.");
                exit(0)
            }

            let read_hits = args[1].to_lowercase() == "-r".to_string() || args[1].to_lowercase() == "--hits";
            if read_hits {
                match proxy_website::read_hits().await {
                    Ok(hits) => {
                        println!("Number of hits: {hits}");
                        exit(0);
                    },
                    Err(e) => {
                        println!("Unable to read number of hits: {e}");
                        exit(1)
                    }
                }
            }

            println!("Argument '{}' not recognized. Use '--help' to see available commands.", args[1]);
            exit(1);
        }


        let redirect_uri = match env::var("REDIRECT_URI") {
            Ok(v) => v,
            Err(e) => {
                println!("Warning!! No 'REDIRECT_URI' Environment Variable found. Defaulting to a placeholder: 'https://en.wikipedia.org/wiki/Leonhard_Euler'.\n
run -- h to see more information on how to set an environment variable.\n
Error: {e}");
                "https://en.wikipedia.org/wiki/Leonhard_Euler".to_string()
            }
        };

        let track_hits : bool = env::var("TRACK_HITS").is_ok();

        Config{
            redirect_uri,
            track_hits,
        }
    }
}

// THIS FUNCTION HANDLES WHAT ACTUALLY HAPPENS WHEN SOMEONE CONNECTS
async fn handle_connect(State(state): State<AppState>) -> Redirect {
    if state.config.track_hits {
        proxy_website::increment_hit_count(&state.pool).await;
    }

    Redirect::to(&state.config.redirect_uri) // Redirect
}

async fn create_app(config: Config) -> Router {
    let pool: Pool<Sqlite> = initalize_db().await.map_or_else(|e| {
        println!("Issue initalizing database: {e}");
        exit(1);
    }, |p| p);

    let state = AppState {
        pool,
        config
    };

    Router::new()
        .route("/", get(handle_connect)).with_state(state)
}

#[tokio::main]
async fn main() {
    let args = env::args().collect();
    let config = Config::new(args).await;
    let app = create_app(config).await;

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
