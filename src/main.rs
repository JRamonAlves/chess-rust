use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use shakmaty::{CastlingMode, Chess, Color, Move, Position, san::San, uci::UciMove};
use thiserror::Error;
use tokio::sync::RwLock;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, decompression::DecompressionLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

type Store = Arc<RwLock<HashMap<Uuid, GameEntry>>>;

#[derive(Clone)]
struct AppState {
    store: Store,
}

#[derive(Debug)]
struct GameEntry {
    pos: Chess,
    history_uci: Vec<String>,
    history_san: Vec<String>,
}

#[derive(Serialize)]
struct GameResponse {
    id: Uuid,
    fen: String,
    legal_moves: Vec<String>,
    moves_uci: Vec<String>,
    moves_san: Vec<String>,
    status: GameStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum GameStatus {
    Ongoing { to_move: SideToMove, in_check: bool },
    Checkmate { winner: SideToMove },
    Stalemate,
    Draw,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum SideToMove {
    White,
    Black,
}

#[derive(Deserialize)]
struct CreateGameRequest {
    // Optional. Start new game from default start position if not provided.
    // Note: Custom FEN parsing is supported; invalid FEN returns 400.
    fen: Option<String>,
}

#[derive(Serialize)]
struct CreateGameResponse {
    id: Uuid,
    fen: String,
}

#[derive(Deserialize)]
struct ApplyMoveRequest {
    // Move in UCI notation, e.g. "e2e4", "g1f3", "e7e8q" (promotion)
    uci: String,
}

#[derive(Serialize)]
struct ApplyMoveResponse {
    id: Uuid,
    applied_uci: String,
    applied_san: String,
    fen: String,
    legal_moves: Vec<String>,
    status: GameStatus,
}

#[derive(Error, Debug)]
enum ApiError {
    #[error("game not found")]
    NotFound,
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("illegal move: {0}")]
    IllegalMove(String),
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::IllegalMove(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/games", post(create_game))
        .route("/games/:id", get(get_game).delete(delete_game))
        .route("/games/:id/moves", get(list_legal_moves).post(apply_move))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(DecompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
    info!("Starting server on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");
    axum::serve(listener, app)
        .await
        .expect("Axum server failed");
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,axum=info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}

async fn root() -> &'static str {
    "Chess REST API. See /health and /games endpoints."
}

async fn health() -> &'static str {
    "ok"
}

async fn create_game(
    State(state): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, ApiError> {
    let id = Uuid::new_v4();

    // Initialize position
    let pos = if let Some(fen_str) = req.fen.as_deref() {
        use shakmaty::fen::Fen;
        let fen: Fen = fen_str
            .parse()
            .map_err(|e| ApiError::BadRequest(format!("invalid FEN: {e}")))?;
        fen.into_position(CastlingMode::Standard)
            .map_err(|e| ApiError::BadRequest(format!("invalid FEN position: {e}")))?
    } else {
        Chess::default()
    };

    let fen = fen_of(&pos);

    let entry = GameEntry {
        pos,
        history_uci: Vec::new(),
        history_san: Vec::new(),
    };

    state.store.write().await.insert(id, entry);

    Ok(Json(CreateGameResponse { id, fen }))
}

async fn get_game(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GameResponse>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&id).ok_or(ApiError::NotFound)?;

    Ok(Json(game_response(id, entry)))
}

async fn delete_game(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut store = state.store.write().await;
    if store.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn list_legal_moves(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<String>>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&id).ok_or(ApiError::NotFound)?;

    let moves = legal_moves_uci(&entry.pos);
    Ok(Json(moves))
}

async fn apply_move(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApplyMoveRequest>,
) -> Result<Json<ApplyMoveResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&id).ok_or(ApiError::NotFound)?;

    // Parse UCI move
    let uci: UciMove = req
        .uci
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("invalid UCI: {e}")))?;

    // Convert UCI to a legal move for current position
    let m: Move = uci
        .to_move(&entry.pos)
        .map_err(|e| ApiError::IllegalMove(format!("{e}")))?;

    // Convert to SAN for history before playing
    let san = San::from_move(&entry.pos, m).to_string();

    // Play the move
    // Ensure to update the position; play_unchecked mutates in-place for Chess.
    entry.pos.play_unchecked(m);

    let fen = fen_of(&entry.pos);
    entry.history_uci.push(uci.to_string());
    entry.history_san.push(san.clone());

    let status = status_of(&entry.pos);
    let legal_moves = legal_moves_uci(&entry.pos);

    Ok(Json(ApplyMoveResponse {
        id,
        applied_uci: req.uci,
        applied_san: san,
        fen,
        legal_moves,
        status,
    }))
}

fn game_response(id: Uuid, entry: &GameEntry) -> GameResponse {
    GameResponse {
        id,
        fen: fen_of(&entry.pos),
        legal_moves: legal_moves_uci(&entry.pos),
        moves_uci: entry.history_uci.clone(),
        moves_san: entry.history_san.clone(),
        status: status_of(&entry.pos),
    }
}

fn legal_moves_uci(pos: &Chess) -> Vec<String> {
    let mode: CastlingMode = pos.castles().mode();
    pos.legal_moves()
        .into_iter()
        .map(|m| m.to_uci(mode).to_string())
        .collect()
}

fn status_of(pos: &Chess) -> GameStatus {
    match pos.outcome() {
        shakmaty::Outcome::Known(known) => match known {
            shakmaty::KnownOutcome::Decisive { winner } => match winner {
                Color::White => GameStatus::Checkmate {
                    winner: SideToMove::White,
                },
                Color::Black => GameStatus::Checkmate {
                    winner: SideToMove::Black,
                },
            },
            shakmaty::KnownOutcome::Draw => {
                if pos.is_stalemate() {
                    GameStatus::Stalemate
                } else {
                    GameStatus::Draw
                }
            }
        },
        shakmaty::Outcome::Unknown => GameStatus::Ongoing {
            to_move: match side_to_move(pos) {
                Color::White => SideToMove::White,
                Color::Black => SideToMove::Black,
            },
            in_check: pos.is_check(),
        },
    }
}

fn side_to_move(pos: &Chess) -> Color {
    // Using a trick: if any legal move exists for White when pos is white to move, but we can directly access via pos.turn()
    // Shakmaty exposes the side to move via pos.turn().
    pos.turn()
}

fn fen_of(pos: &Chess) -> String {
    // Prefer a legal en-passant encoding for a precise state.
    // Fen implements Display.
    // This uses the default "Legal" en passant mode to not include pseudo squares.
    use shakmaty::{EnPassantMode, fen::Fen};
    Fen::from_position(pos, EnPassantMode::Legal).to_string()
}

// ----- Tests (basic) -----
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::Position;

    #[test]
    fn default_game_has_20_legal_moves() {
        let pos = Chess::default();
        assert_eq!(pos.legal_moves().len(), 20);
    }

    #[test]
    fn uci_parsing_and_play() {
        let mut entry = GameEntry {
            pos: Chess::default(),
            history_uci: vec![],
            history_san: vec![],
        };
        let uci: UciMove = "e2e4".parse().expect("uci");
        let m = uci.to_move(&entry.pos).expect("legal");
        entry.pos.play_unchecked(m);
        assert_eq!(entry.history_uci.len(), 0);
        assert_eq!(entry.pos.legal_moves().len(), 20); // after e4, still many moves
    }
}
