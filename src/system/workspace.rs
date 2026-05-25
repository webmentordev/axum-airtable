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
pub struct WorkspaceRecord {
    pub unique_id: Option<String>,
    pub title: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspaceResponse {
    pub unique_id: String,
    pub title: String,
    pub position: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn get_workspaces(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_uid): Path<String>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
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

    match sqlx::query_as::<_, WorkspaceResponse>(
        "SELECT unique_id, title, position, created_at, updated_at FROM workspaces WHERE app_id = $1",
    )
    .bind(app_uid)
    .fetch_all(&state.pool)
    .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(json!({
                "message": "Workspaces have been fetched!",
                "data": records
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to fetch workspace"
            })),
        ),
    }
}

pub async fn create_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_uid): Path<String>,
    Json(payload): Json<WorkspaceRecord>,
) -> impl IntoResponse {
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Title is required"
            })),
        );
    }

    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
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

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to start transaction"
                })),
            );
        }
    };

    let workspace_uid = generate_id("wrk_");

    let workspace = match sqlx::query_scalar::<_, i32>(
        "INSERT INTO workspaces (unique_id, app_id, title, position) VALUES ($1, $2, $3, $4) RETURNING id",
        
    ).bind(&workspace_uid).bind(&app_uid).bind(&payload.title).bind(payload.position)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(workspace) => workspace,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to create workspace"
                })),
            );
        }
    };

    let system_fields = vec![
        ("Created At", "created_at"),
        ("Updated At", "updated_at"),
    ];

    for (index, (title, field_type)) in system_fields.iter().enumerate() {
        if let Err(_) = sqlx::query(
            "INSERT INTO fields (workspace_id, unique_id, title, field_type, is_system, position) VALUES ($1, $2, $3, $4, true, $5)").bind(workspace).bind(generate_id("fld_")).bind(title).bind(field_type).bind( index as i32)
        .execute(&mut *tx)
        .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to create system fields"
                })),
            );
        }
    }

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
            "message": "Workspace has been created!",
            "data": {
                "unique_id": workspace_uid,
                "title": payload.title,
                "position": payload.position
            }
        })),
    )
}

pub async fn update_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((app_uid, workspace_uid)): Path<(String, String)>,
    Json(payload): Json<WorkspaceRecord>,
) -> impl IntoResponse {
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Title or position is missing"
            })),
        );
    }

    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
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

    let mut tx = match state.pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Failed to start transaction" })),
            );
        }
    };

    match sqlx::query(
        "UPDATE workspaces SET title = $1, position = $2, updated_at = $3 WHERE unique_id = $4",
    )
    .bind(&payload.title)
    .bind(&payload.position)
    .bind(chrono::Utc::now().naive_utc())
    .bind(&workspace_uid)
    .execute(&mut *tx)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            if let Err(_) = tx.commit().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": "Failed to commit transaction"
                    })),
                );
            }
            (
                StatusCode::OK,
                Json(json!({ "message": "Workspace has been updated" })),
            )
        }
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Workspace not found!" })),
        ),
        Err(e) => {
            println!("{}", e.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            )
        }
    }
}

pub async fn delete_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((app_uid, workspace_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
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

    match sqlx::query("DELETE from workspaces WHERE app_id = $1 AND unique_id = $2")
        .bind(&app_uid)
        .bind(&workspace_uid)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("Workspace {} has been deleted!", workspace_uid) })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Unable to delete the workspace" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}
