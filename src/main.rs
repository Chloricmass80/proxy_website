use std::{process::exit, env};

use sqlx::{Sqlite};
use tokio::net;
use axum::{Router, extract::{State}, response::Redirect, routing::get};

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
    bind_addr: String,
}

impl Config {
    async fn new(args: Vec<String>) -> Self {
        match proxy_website::load_env_variables(){
            Ok(_) => {},
            Err(e) => {
                if *&e.not_found() {
                    eprintln!("Could not find .env file.
If executing the binary create a .env file by name '.env' or 'local.env' inside the binary's executable and fill out the variables.
If executing the crate with cargo create a .env file by name 'local.env' inside the config/ directory.
Falling back to defaults.");
                } else {
                    eprintln!(".env file likely exists but could not be opened: {e}.
Falling back to defaults.");
                }
            }
        }

        if args.get(1).is_some() {
            let help = args[1].to_lowercase() == "-h".to_string() || args[1].to_lowercase() == "--help".to_string();
            if help {
                println!("proxy_website help:
This binary is meant to run on a server as a backend.
It tracks the number of hits it gets, then immediately redirects\n
To read hits, add an argument '-r' or '--hits'. This will not run the server.
Environment Variable setup:
Find environment variables in 'local.env' or '.env'.
To see what environment variables exist, visit example.env.
To set a bool environment variable to false, simply remove its decleration.
To set a bool environment variable to true, add its decleration to equal any value.

To reset the number of hits to 0, navigate to the executable's directory and remove the 'visitors.db' file.");
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
                        eprintln!("Unable to read number of hits: {e}");
                        exit(1)
                    }
                }
            }

            println!("Argument '{}' not recognized. Use '--help' to see available commands.", args[1]);
            exit(0);
        }


        let redirect_uri = match env::var("REDIRECT_URI") {
            Ok(v) => v,
            Err(e) => {
                println!("Warning!! No 'REDIRECT_URI' Environment Variable found. Defaulting to a placeholder: 'https://en.wikipedia.org/wiki/Leonhard_Euler'.
run -- h to see more information on how to set an environment variable.
Error: {e}");
                "https://en.wikipedia.org/wiki/Leonhard_Euler".to_string()
            }
        };

        let track_hits : bool = env::var("TRACK_HITS").is_ok();
        
        let bind_addr : String = match env::var("BIND_ADDR") {
            Ok(v) => v,
            Err(e) => {
                println!("Warning!! No 'BIND_ADDR' Environment Variable found. Defaulting to a placeholder: '0.0.0.0:3000'.
run -- h to see more information on how to set an environment variable.
Error: {e}");
                "0.0.0.0:3000".to_string()
                }
            };

        Config{
            redirect_uri,
            track_hits,
            bind_addr,
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
    let pool = match initalize_db().await{
        Ok(v) => v,
        Err(e) => {
            eprintln!("Issue initalizing database: {e}");
            exit(1);
        }
    };

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

    let listener = match net::TcpListener::bind(&config.bind_addr).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not start on port: {e}");
            exit(1);
        }
    };

    let app = create_app(config).await;

    match listener.local_addr() {
        Ok(addr) => {println!("Server running on http://localhost:{}", addr.port());},
        Err(e) => {println!("Server running. (Could not generate http socket address: {e})");}
    }

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Could not serve: {e}");
        exit(1)
    }
}
