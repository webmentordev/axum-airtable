use crate::AuthUser;
use crate::Response;
use crate::database::AppState;
use crate::utils::generate_id;

use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RecordForm {
    pub unique_id: Option<String>,
    pub title: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RecordResponse {
    pub unique_id: String,
    pub title: String,
    pub position: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn get_system_records(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Records fetched!"
        })),
    )
}

pub async fn create_system_record(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
    Json(payload): Json<RecordResponse>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "New record inserted!"
        })),
    )
}

pub async fn update_system_record(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, record_uid)): Path<(String, String)>,
    Json(payload): Json<RecordResponse>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Updated a record"
        })),
    )
}

pub async fn delete_system_record(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, record_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Deleted a record!" })),
    )
}
