use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};

type KVData = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    let app = Router::new()
        .route("/", get(health_check))
        .route("/value", post(set_value))
        .route("/value", get(get_value))
        .route("/value", delete(delete_value))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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

async fn set_value(state: State<KVData>, Json(body): Json<SetValueRequest>) -> impl IntoResponse {
    let mut state = state.lock().unwrap();

    state.insert(body.key, body.value);

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

async fn get_value(state: State<KVData>, Query(body): Query<GetValueRequest>) -> impl IntoResponse {
    let state = state.lock().unwrap();

    if let Some(value) = state.get(&body.key) {
        Response::builder()
            .status(StatusCode::OK)
            .body(
                serde_json::to_string(&GetValueResponse {
                    value: value.clone(),
                })
                .unwrap_or_default(),
            )
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
    state: State<KVData>,
    Query(body): Query<DeleteValueRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();

    if state.remove(&body.key).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
