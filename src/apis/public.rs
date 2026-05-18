use crate::Response;
use axum::{Json, extract::Path};
use rand::{Rng, distributions::Alphanumeric};

pub async fn get_records(Path((app, workspace)): Path<(String, String)>) -> Json<Response> {
    // let records = sqlx::query_as();
    Json(Response {
        message: format!("Response from GET AppID {} & Workspace {}", app, workspace),
    })
}

pub async fn get_record(Path(id): Path<String>) -> Json<Response> {
    Json(Response {
        message: format!("Response from GET a RecordID {}", id),
    })
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

pub fn generate_id(prefix: &str) -> String {
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(17)
        .map(char::from)
        .collect();

    format!("{}{}", prefix, random)
}
