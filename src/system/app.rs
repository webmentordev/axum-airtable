use crate::database::AppState;
use crate::{AuthUser, utils::generate_id};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct AppRecord {
    pub unique_id: Option<String>,
    pub title: String,
}

pub async fn get_apps(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppRecord>("SELECT unique_id, title FROM apps WHERE owner_id = $1")
        .bind(user_id)
        .fetch_all(&state.pool)
        .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(json!({
                "message": "Apps have been fetched!",
                "data": records
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to fetch apps"
            })),
        ),
    }
}

pub async fn create_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<AppRecord>,
) -> impl IntoResponse {
    match sqlx::query("INSERT into apps (owner_id, unique_id, title) VALUES ($1, $2, $3)")
        .bind(&user_id)
        .bind(generate_id("app"))
        .bind(&body.title)
        .execute(&state.pool)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "message": format!("App has been created!")
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to create the new app"
            })),
        ),
    }
}

pub async fn get_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppRecord>(
        "SELECT unique_id, title FROM apps WHERE unique_id = $1 AND owner_id = $2",
    )
    .bind(&uid)
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => (
            StatusCode::OK,
            Json(json!({
                "message": "App has been fetched!",
                "data": row
            })),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "App not found!"
            })),
        ),
    }
}

pub async fn update_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Json(payload): Json<AppRecord>,
) -> impl IntoResponse {
    match sqlx::query("UPDATE apps SET title = $1 WHERE unique_id = $2 AND owner_id = $3")
        .bind(&payload.title)
        .bind(&uid)
        .bind(&user_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": format!("App {} title has been updated!", uid)
            })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "App not found!" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}

pub async fn delete_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE from apps WHERE unique_id = $1 AND owner_id = $2")
        .bind(&uid)
        .bind(&user_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("App {} has been deleted!", uid) })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "App not found!" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}
