use axum::http::StatusCode;
use axum::{
    extract::Query,
    extract::State,
    routing::{get, post},
    Json, Router,
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_yaml;
use sqlx::PgPool;
use std::io::read_to_string;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Deserialize)]
struct OptParams {
    player: Option<String>,
    certain_game: Option<String>,
    limit_past_games: Option<i64>,
}

#[derive(serde::Serialize, sqlx::FromRow, Debug)]
struct TeamResponse {
    team_name: String,
    team_id: i64,
    game_date: NaiveDateTime,
    matchup: String,
    game_id: String,
    wl: String,
    fga: i64,
    fgm: i64,
    fg_pct: f64,
    fgthree_m: i64,
    fgthree_a: i64,
    fgthree__pct: f64,
    oreb: i64,
    dreb: i64,
    tov: i64,
    ftm: i64,
    fta: i64,
    ft_pct: f64,
}

#[derive(serde::Serialize, sqlx::FromRow, Debug)]
struct PlayerResponse {
    player: String,
    min: f64,
    team: String,
    game_id: String,
    game_date: NaiveDateTime,
    fgm: f64,
    fga: f64,
    fg_pct: f64,
    fgthree_m: f64,
    fgthree_a: f64,
    fgthree__pct: f64,
    ftm: f64,
    fta: f64,
    ft_pct: f64,
    oreb: f64,
    dreb: f64,
    ast: f64,
    stl: f64,
    blk: f64,
    turnovers: f64,
    plus_minus: f64,
}

#[derive(Deserialize)]
struct Config {
    username: String,
    password: String,
    host: String,
    port: String,
    db_port: u16,
    db: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut contents = String::new();
    let mut file = fs::File::open("config.yaml").await?;
    file.read_to_string(&mut contents).await?;

    let config: Config = serde_yaml::from_str(&contents)?;
    let listener = TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
    println!("Listening on {}:{}", &config.host, &config.port);

    let connection = sqlx::postgres::PgConnectOptions::new()
        .host(&config.host)
        .port(config.db_port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.db);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connection)
        .await?;

    let state = AppState { db: pool };

    let route = Router::new()
        .route("/nba-bet/team", get(team_data))
        .route("/nba-bet/player", get(player_data))
        .with_state(state);
    axum::serve(listener, route).await.unwrap();

    Ok(())
}

async fn team_data(State(state): State<AppState>) -> Result<Json<Vec<TeamResponse>>, StatusCode> {
    let results: Vec<TeamResponse> = sqlx::query_as::<_, TeamResponse>(
        "
        select 
            team_name,
            team_id,
            game_date,
            matchup,
            game_id,
            wl,
            fga,
            fgm,
            fg_pct,
            fgthree_m,
            fgthree_a,
            fgthree__pct,
            oreb,
            dreb,
            tov,
            ftm,
            fta,
            ft_pct 
        from clean_team_data",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Team error: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(results))
}

async fn player_data(
    State(state): State<AppState>,
    Query(param): Query<OptParams>,
) -> Result<Json<Vec<PlayerResponse>>, StatusCode> {
    let limit = param.limit_past_games.unwrap_or(3);
    let player = param.player.unwrap_or("".to_string());
    let certain_game = param.certain_game.unwrap_or("".to_string());

    let results: Vec<PlayerResponse> = sqlx::query_as::<_, PlayerResponse>(
        "
with last_n as(
    select
    game_id,
    player,
    row_number() over(partition by player order by game_date desc) as rn
from clean_player_data
)
select
clean_player_data.player,
min,
team,
clean_player_data.game_id,
game_date,
fgm,
fga,
fg_pct,
fgthree_m,
fgthree_a,
fgthree__pct,
ftm,
fta,
ft_pct,
oreb,
dreb,
ast,
stl,
blk,
turnovers,
plus_minus
from clean_player_data
inner join last_n 
    on last_n.player = clean_player_data.player
    and rn <= $1
    and last_n.game_id = clean_player_data.game_id
where ($2 = '' or clean_player_data.player=$2) and
    ($3 = '' or clean_player_data.game_id=$3)
",
    )
    .bind(limit)
    .bind(player)
    .bind(certain_game)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("player error: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(results))
}
