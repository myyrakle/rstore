use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};

type KVData = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    let app = Router::new()
        .route("/", get(health_check))
        .route("/value", post(set_value))
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
