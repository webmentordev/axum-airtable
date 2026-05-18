mod apis;
mod auth;
mod database;
use apis::private::*;
use apis::public::*;
use auth::routes::*;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Response {
    message: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
    record_limit: u32,
}

#[tokio::main]
async fn main() {
    let state_data = database::setup_database().await;
    let state = AppState {
        pool: state_data.0,
        record_limit: state_data.1,
    };
    let api_routes = Router::new()
        .route(
            "/records/{app}/{workspace}",
            get(get_records).post(create_record),
        )
        .route(
            "/record/{id}",
            get(get_record).delete(delete_record).patch(update_record),
        );
    let app_routes = Router::new().route("/records", get(app_records));
    let auth_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/signup", post(signup_handler))
        .route("/logout", post(logout_handler));
    let app = Router::new()
        .route("/health", get(get_health))
        .nest("/api", api_routes)
        .nest("/app", app_routes)
        .nest("/auth", auth_routes)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Server is running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_health() -> Json<Response> {
    Json(Response {
        message: "System is running!".to_string(),
    })
}
