use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(health_check));

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
