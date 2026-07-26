use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json
};
use axum::http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(serde::Serialize,sqlx::FromRow)]
struct SqlResponse {
    team: String
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


async fn team_data(State(state): State<AppState>)->Result<Json<Vec<SqlResponse>>,StatusCode> {
    let results: Vec<SqlResponse> = sqlx::query_as::<_,SqlResponse>("select team from analytics.clean_team_data").fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

async fn player_data() {}
