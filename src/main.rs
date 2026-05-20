mod apis;
mod auth;
mod database;
mod system;
mod utils;

use apis::routes::*;
use auth::*;
use database::*;
use system::app::*;
use system::workspace::*;

use axum::{
    Json, Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::Serialize;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();
    let token = env::var("JWT_SECRET").expect("JWT_SECRET not found!");
    if token.len() < 30 {
        panic!("JWT_TOKEN must be 30 characters long.");
    }

    let state = database::setup_database().await;

    let api_routes = Router::new()
        .route(
            "/records/{app_uid}/{workspace_uid}",
            get(get_records).post(create_record),
        )
        .route(
            "/record/{uid}",
            get(get_record).delete(delete_record).patch(update_record),
        );
    let app_routes = Router::new()
        .route("/", get(get_apps).post(create_app))
        .route(
            "/app/{uid}",
            get(get_app).patch(update_app).delete(delete_app),
        );
    let workspace_rotues = Router::new()
        .route("/", get(get_workspaces).post(create_workspace))
        .route(
            "/workspace/{uid}",
            get(get_workspace)
                .patch(update_workspace)
                .delete(delete_workspace),
        );

    let system_routes = Router::new()
        .nest("/apps", app_routes)
        .nest("/workspaces", workspace_rotues);

    let auth_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/signup", post(signup_handler))
        .route("/logout", post(logout_handler));

    let app = Router::new()
        .route("/health", get(get_health))
        .nest("/api", api_routes)
        .nest("/system", system_routes)
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
