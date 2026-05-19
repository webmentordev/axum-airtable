use crate::AuthUser;
use crate::Response;

use axum::{Json, extract::Path};

pub async fn app_records(
    AuthUser(_email): AuthUser,
    Path(workspace): Path<String>,
) -> Json<Response> {
    Json(Response {
        message: format!("App Response from GET Workspace {}", workspace),
    })
}
