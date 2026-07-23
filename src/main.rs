use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize)]
struct SqlResponse {
    table: String,
    data: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on 127.0.0.1:8080");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("")
        .await?;

    let state = AppState { db: pool };

    let route = Router::new()
        .route("/nba-bet/team", get(team_data))
        .route("/nba-bet/player", get(player_data))
        .with_state(state);
    axum::serve(listener, route).await.unwrap();

    Ok(())
}

async fn team_data(State(state): State<AppState>) {}

async fn player_data() {}
