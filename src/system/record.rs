use crate::AuthUser;
use crate::database::AppState;
use crate::utils::generate_id;

use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use serde_json::json;

#[derive(sqlx::FromRow)]
struct FieldRow {
    unique_id: String,
    title: String,
    field_type: String,
    position: i32,
    is_system: bool,
    settings: Option<serde_json::Value>,
}

#[derive(sqlx::FromRow)]
struct CellRow {
    row_id: String,
    field_id: Option<String>,
    field_type: Option<String>,
    value_text: Option<String>,
    value_number: Option<f64>,
    value_boolean: Option<bool>,
    value_date: Option<NaiveDateTime>,
    value_json: Option<serde_json::Value>,
}

pub async fn get_system_records(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
) -> impl IntoResponse {
    let (app_uid, workspace_id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Workspace not found" })),
            );
        }
    };
    if sqlx::query_scalar::<_, i32>("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Records not found." })),
        );
    }

    let fields = sqlx::query_as::<_, FieldRow>(
        "SELECT unique_id, title, field_type, position, is_system, settings
         FROM fields
         WHERE workspace_id = $1
         ORDER BY position ASC",
    )
    .bind(workspace_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let raw = sqlx::query_as::<_, CellRow>(
        "SELECT
             r.unique_id  AS row_id,
             r.created_at AS row_created_at,
             f.unique_id  AS field_id,
             f.field_type,
             c.value_text,
             c.value_number,
             c.value_boolean,
             c.value_date,
             c.value_json
         FROM rows r
         LEFT JOIN cells c ON c.row_id = r.unique_id
         LEFT JOIN fields f ON f.unique_id = c.field_id
         WHERE r.workspace_id = $1
         ORDER BY r.id ASC, f.position ASC",
    )
    .bind(workspace_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut row_map: IndexMap<String, serde_json::Value> = IndexMap::new();

    for cell in raw {
        let entry = row_map
            .entry(cell.row_id.clone())
            .or_insert_with(|| json!({ "id": cell.row_id }));

        if let (Some(field_id), Some(field_type)) = (&cell.field_id, &cell.field_type) {
            let col_name = fields
                .iter()
                .find(|f| &f.unique_id == field_id)
                .map(|f| f.title.clone())
                .unwrap_or_else(|| field_id.clone());

            let value = resolve_cell_value(&field_type, &cell);
            entry[col_name] = value;
        }
    }

    let records: Vec<serde_json::Value> = row_map.into_values().collect();

    (
        StatusCode::OK,
        Json(json!({
            "fields": fields.iter().map(|f| json!({
                "id": f.unique_id,
                "title": f.title,
                "type": f.field_type,
                "position": f.position,
                "is_system": f.is_system,
                "settings": f.settings
            })).collect::<Vec<_>>(),
            "records": records,
        })),
    )
}

fn resolve_cell_value(field_type: &str, cell: &CellRow) -> serde_json::Value {
    match field_type {
        "text" | "email" => json!(cell.value_text),
        "number" => json!(cell.value_number),
        "checkbox" => json!(cell.value_boolean.unwrap_or(false)),
        "date" | "created_at" | "updated_at" => {
            json!(cell.value_date.map(|d| d.to_string()))
        }
        "multi_select" | "attachments" => cell.value_json.clone().unwrap_or(json!([])),
        _ => cell
            .value_json
            .clone()
            .or_else(|| cell.value_text.as_ref().map(|t| json!(t)))
            .unwrap_or(json!(null)),
    }
}

pub async fn create_system_record(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(workspace_uid): Path<String>,
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
                StatusCode::NOT_FOUND,
                Json(json!({
                    "message": "Record not found."
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

    let row_uid = match sqlx::query_scalar::<_, String>(
        "INSERT INTO rows (workspace_id, unique_id, created_by) VALUES ($1, $2, $3) RETURNING unique_id",
    )
    .bind(&id)
    .bind(generate_id("rec_"))
    .bind(&user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            );
        }
    };

    let data = match sqlx::query_as::<_, (String, String)>(
        "SELECT unique_id, field_type FROM fields WHERE workspace_id = $1",
    )
    .bind(&id)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            );
        }
    };

    for (uid, f_type) in data {
        match f_type.to_lowercase().as_str() {
            "text" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_text) VALUES ($1, $2, '')",
                )
                .bind(&row_uid)
                .bind(&uid)
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            "email" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_text) VALUES ($1, $2, '')",
                )
                .bind(&row_uid)
                .bind(&uid)
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            "number" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_number) VALUES ($1, $2, 0.00)",
                )
                .bind(&row_uid)
                .bind(&uid)
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            "checkbox" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_boolean) VALUES ($1, $2, 0)",
                )
                .bind(&row_uid)
                .bind(&uid)
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            "date" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_date) VALUES ($1, $2, null)",
                )
                .bind(&row_uid)
                .bind(&uid)
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            "created_at" | "updated_at" => {
                if let Err(_) = sqlx::query(
                    "INSERT into cells (row_id, field_id, value_date) VALUES ($1, $2, $3)",
                )
                .bind(&row_uid)
                .bind(&uid)
                .bind(chrono::Utc::now().naive_utc())
                .execute(&mut *tx)
                .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Can not insert data" })),
                    );
                }
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Unknown data type" })),
                );
            }
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
        StatusCode::OK,
        Json(json!({
            "message": "New record inserted!"
        })),
    )
}

pub async fn delete_system_record(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((workspace_uid, record_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (app_uid, id) = match sqlx::query_as::<_, (String, i32)>(
        "SELECT app_id, id FROM workspaces WHERE unique_id = $1",
    )
    .bind(&workspace_uid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "message": "Fields not found."
                })),
            );
        }
    };
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

    match sqlx::query("DELETE FROM rows WHERE unique_id = $1 AND workspace_id = $2")
        .bind(&record_uid)
        .bind(&id)
        .execute(&state.pool)
        .await
    {
        Ok(record) if record.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": "Record has been deleted!",
            })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Record not found."
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
