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
pub struct FieldResponse {
    pub unique_id: String,
    pub title: String,
    pub field_type: String,
    pub position: i32,
    pub is_system: bool,
    pub settings: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct FieldRequest {
    pub title: String,
    pub field_type: String,
    pub position: i32,
    pub settings: Option<serde_json::Value>,
}

pub async fn get_fields(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
) -> impl IntoResponse {
    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            );
        }
    };

    if let Err(e) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "message": "Access denied" })),
        );
    }

    match sqlx::query_as::<_, FieldResponse>("SELECT unique_id, title, field_type, position, is_system, settings FROM fields WHERE workspace_id = $1").bind(id).fetch_all(&state.pool).await{
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "message": "Fields have been fetched!",
                "data": result
            })),
        ),
        Err(e) => {
            println!("{}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            )
        }
    }
}

pub async fn create_field(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
    Json(payload): Json<FieldRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            );
        }
    };

    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            );
        }
    };

    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&mut *tx)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "message": "Access denied" })),
        );
    }

    match sqlx::query("INSERT INTO fields (workspace_id, unique_id, title, field_type, position, settings) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(generate_id("fld_"))
        .bind(&payload.title)
        .bind(&payload.field_type)
        .bind(&payload.position)
        .bind(&payload.settings)
        .execute(&mut *tx)
        .await
    {
        Ok(_) => {
            if let Err(_) = tx.commit().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": "Failed to commit transaction"
                    })),
                );
            }
            (
                StatusCode::CREATED,
                Json(json!({
                    "message": "Field has been added!",
                })),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to create system fields"
            })),
        )
    }
}

pub async fn get_field(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, field_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            );
        }
    };
    if let Err(e) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "message": "Access denied" })),
        );
    }

    match sqlx::query_as::<_, FieldResponse>("SELECT unique_id, title, field_type, position, is_system, settings FROM fields WHERE unique_id = $1").bind(&field_uid).fetch_one(&state.pool).await{
        Ok(record) => (
            StatusCode::OK,
                Json(json!({
                    "message": "Field fetched!",
                    "data": record
                })),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to fetch the field"
            })),
        )
    }
}

pub async fn update_field(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, field_uid)): Path<(String, String)>,
    Json(payload): Json<FieldRequest>,
) -> impl IntoResponse {
    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            );
        }
    };
    if let Err(e) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "message": "Access denied" })),
        );
    }

    match sqlx::query("UPDATE fields SET title = $1, field_type = $2, position = $3, settings = $4 WHERE unique_id = $5 AND is_system = false")
        .bind(&payload.title).bind(&payload.field_type).bind(&payload.position).bind(&payload.settings).bind(&field_uid)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": "Field updated!",
            })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to update the field"
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal server error"
            })),
        ),
    }
}

pub async fn delete_field(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, field_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "Failed to fetch fields"
                })),
            );
        }
    };
    if let Err(e) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "message": "Access denied" })),
        );
    }

    match sqlx::query("DELETE FROM fields WHERE unique_id = $1 AND is_system = false")
        .bind(&field_uid)
        .execute(&state.pool)
        .await
    {
        Ok(record) if record.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": "Field has been deleted!",
            })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to delete the field"
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal server error"
            })),
        ),
    }
}
