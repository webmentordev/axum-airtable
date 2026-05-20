use crate::AuthUser;
use crate::Response;

use axum::{Json, extract::Path};

pub async fn get_workspaces(AuthUser(user_id): AuthUser) -> Json<Response> {
    Json(Response {
        message: format!("GET workspaces - {}", user_id),
    })
}

pub async fn create_workspace(AuthUser(user_id): AuthUser) -> Json<Response> {
    Json(Response {
        message: format!("CREATE workspace - {}", user_id),
    })
}

pub async fn get_workspace(AuthUser(user_id): AuthUser, Path(id): Path<String>) -> Json<Response> {
    Json(Response {
        message: format!("GET single workspace at {} - {}", id, user_id),
    })
}

pub async fn update_workspace(
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Json<Response> {
    Json(Response {
        message: format!("UPDATE/PATCH single workspace at {} - {}", id, user_id),
    })
}

pub async fn delete_workspace(
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Json<Response> {
    Json(Response {
        message: format!("DELETE single workspace at {} - {}", id, user_id),
    })
}
