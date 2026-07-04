use crate::{CellRow, FieldRow, Pagination, resolve_cell_value};
use crate::{Response, database::AppState};

use axum::{
    Json,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use indexmap::IndexMap;
use serde_json::json;

pub struct VerifyToken(pub String);

impl<S> FromRequestParts<S> for VerifyToken
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
        Ok(VerifyToken(token.to_string()))
    }
}

pub async fn get_records(
    VerifyToken(api_token): VerifyToken,
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    Path((app, workspace)): Path<(String, String)>,
) -> impl IntoResponse {
    let workspace_id = match sqlx::query_scalar::<_, i32>(
        "SELECT id FROM workspaces WHERE unique_id = $1 AND app_id = $2",
    )
    .bind(&workspace)
    .bind(&app)
    .fetch_one(&state.pool)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "AppID or Workspace not found."
                })),
            );
        }
    };

    if sqlx::query("SELECT app_id FROM tokens WHERE app_id = $1 AND token = $2")
        .bind(&app)
        .bind(&api_token)
        .fetch_one(&state.pool)
        .await
        .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Unauthorized token."
            })),
        );
    }

    let total_records =
        match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM rows WHERE workspace_id = $1")
            .bind(&workspace_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(records) => records,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "message": "Records not found." })),
                );
            }
        };

    let record_per_page = state.record_limit as i64;
    let total_pages = ((total_records as f64) / (record_per_page as f64)).ceil() as i32;
    let off_set = (pagination.page - 1) * record_per_page;

    let fields = sqlx::query_as::<_, FieldRow>(
        "SELECT unique_id, title, field_type, position, is_system, settings
     FROM fields
     WHERE workspace_id = $1
     ORDER BY position ASC",
    )
    .bind(&workspace_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let raw = sqlx::query_as::<_, CellRow>(
        "SELECT
         r.unique_id  AS row_id,
         r.created_at AS row_created_at,
         r.updated_at AS updated_at,
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
     WHERE r.workspace_id = $1 AND r.unique_id IN (
         SELECT unique_id FROM rows WHERE workspace_id = $1 
         ORDER BY id ASC LIMIT $2 OFFSET $3
     )
     ORDER BY r.id ASC, f.position ASC",
    )
    .bind(&workspace_id)
    .bind(record_per_page)
    .bind(off_set)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut row_map: IndexMap<String, serde_json::Value> = IndexMap::new();

    for cell in raw {
        let entry = row_map
            .entry(cell.row_id.clone())
            .or_insert_with(|| json!({ "id": cell.row_id, "updated_at": cell.updated_at }));
        if let (Some(field_id), Some(field_type)) = (&cell.field_id, &cell.field_type) {
            if matches!(field_type.as_str(), | "updated_at") {
                continue;
            }
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
            "total_records": total_records,
            "total_pages": total_pages,
            "records": records,
        })),
    )
}

pub async fn get_record(
    VerifyToken(api_token): VerifyToken,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row = match sqlx::query_as::<_, (i64, i32, String)>(
        "SELECT r.id, r.workspace_id, w.app_id
     FROM rows r
     JOIN workspaces w ON w.id = r.workspace_id
     WHERE r.unique_id = $1",
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Record not found." })),
            );
        }
    };

    let (_, workspace_id, app_id) = row;

    if sqlx::query("SELECT app_id FROM tokens WHERE app_id = $1 AND token = $2")
        .bind(&app_id)
        .bind(&api_token)
        .fetch_one(&state.pool)
        .await
        .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Unauthorized token." })),
        );
    }

    let fields = sqlx::query_as::<_, FieldRow>(
        "SELECT unique_id, title, field_type, position, is_system, settings
         FROM fields
         WHERE workspace_id = $1
         ORDER BY position ASC",
    )
    .bind(&workspace_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let raw = sqlx::query_as::<_, CellRow>(
        "SELECT
             r.unique_id  AS row_id,
             r.created_at AS row_created_at,
             r.updated_at AS updated_at,
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
         WHERE r.unique_id = $1
         ORDER BY f.position ASC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut row_map: IndexMap<String, serde_json::Value> = IndexMap::new();

    for cell in raw {
        let entry = row_map
            .entry(cell.row_id.clone())
            .or_insert_with(|| json!({ "id": cell.row_id, "updated_at": cell.updated_at }));
        if let (Some(field_id), Some(field_type)) = (&cell.field_id, &cell.field_type) {
            if matches!(field_type.as_str(), | "updated_at") {
                continue;
            }
            let col_name = fields
                .iter()
                .find(|f| &f.unique_id == field_id)
                .map(|f| f.title.clone())
                .unwrap_or_else(|| field_id.clone());

            let value = resolve_cell_value(&field_type, &cell);
            entry[col_name] = value;
        }
    }

    let record = row_map.into_values().next();

    match record {
        Some(rec) => (StatusCode::OK, Json(json!({ "record": rec }))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Record not found." })),
        ),
    }
}

pub async fn create_record(Path((app, workspace)): Path<(String, String)>) -> Json<Response> {
    Json(Response {
        message: format!("Response from POST AppID {} & Workspace {}", app, workspace),
    })
}

pub async fn delete_record(Path(id): Path<String>) -> Json<Response> {
    Json(Response {
        message: format!("Response from DELETE RecordID {}", id),
    })
}

pub async fn update_record(Path(id): Path<String>) -> Json<Response> {
    Json(Response {
        message: format!("Response from PATCH RecordID {}", id),
    })
}
