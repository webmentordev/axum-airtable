use crate::Response;
use axum::{Json, extract::Path};

pub async fn app_records(Path((app, workspace)): Path<(String, String)>) -> Json<Response> {
    Json(Response {
        message: format!(
            "App resposne from GET AppID {} & Workspace {}",
            app, workspace
        ),
    })
}
