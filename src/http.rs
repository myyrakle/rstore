mod engine;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use engine::KVEngine;

#[tokio::main]
async fn main() {
    let engine = KVEngine::new();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/value", post(set_value))
        .route("/value", get(get_value))
        .route("/value", delete(delete_value))
        .route("/clear", delete(clear_all))
        .with_state(engine);

    let addr = "0.0.0.0:13535";
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .body("OK".to_string())
        .unwrap()
}

#[derive(serde::Deserialize)]
struct SetValueRequest {
    key: String,
    value: String,
}

async fn set_value(
    engine: State<KVEngine>,
    Json(body): Json<SetValueRequest>,
) -> impl IntoResponse {
    let result = engine.set_key_value(body.key, body.value);

    if result.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}

#[derive(serde::Deserialize)]
struct GetValueRequest {
    key: String,
}

#[derive(serde::Serialize)]
struct GetValueResponse {
    value: String,
}

async fn get_value(
    engine: State<KVEngine>,
    Query(body): Query<GetValueRequest>,
) -> impl IntoResponse {
    if let Ok(value) = engine.get_key_value(&body.key) {
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string(&GetValueResponse { value }).unwrap_or_default())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Key not found".to_string())
            .unwrap()
    }
}

#[derive(serde::Deserialize)]
struct DeleteValueRequest {
    key: String,
}

async fn delete_value(
    engine: State<KVEngine>,
    Query(body): Query<DeleteValueRequest>,
) -> impl IntoResponse {
    let result = engine.delete_key_value(&body.key);

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => match error {
            engine::KVError::KeyNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

async fn clear_all(state: State<KVEngine>) -> impl IntoResponse {
    let result = state.clear_all();

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
