use std::path::Path;

use axum::extract::Query;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;

use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct BookmarkQuery {
    force: Option<String>,
    url: Option<String>,
}

const CACHE_DIR: &str = ".cache";
const CACHE_FILE: &str = ".cache/bookmarks.json";

async fn save_cache(bytes: &[u8]) -> Result<(), ApiError> {
    if !Path::new(CACHE_DIR).exists() {
        fs::create_dir_all(CACHE_DIR).await?;
    }
    let mut f = fs::File::create(CACHE_FILE).await?;
    f.write_all(bytes).await?;
    Ok(())
}

async fn load_cache() -> Result<Option<Vec<u8>>, ApiError> {
    if !Path::new(CACHE_FILE).exists() {
        return Ok(None);
    }
    let bytes = fs::read(CACHE_FILE).await?;
    Ok(Some(bytes))
}

pub async fn serve_bookmarks(
    State(state): State<AppState>,
    Query(q): Query<BookmarkQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let as_json_response = |body: Vec<u8>| {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        (StatusCode::OK, headers, body)
    };

    let force_yes = matches!(q.force.as_deref(), Some("yes"));

    if !force_yes && let Some(bytes) = load_cache().await? {
        return Ok(as_json_response(bytes));
    }
    // cache miss -> fallthrough to fetch

    let url = q.url.as_deref().unwrap_or("").trim();
    if url.is_empty() {
        // 指示により 500 系で返す
        return Err(ApiError::Internal);
    }

    let resp = state
        .http
        .get(url)
        .header(reqwest::header::USER_AGENT, "private_start_page/0.1")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(ApiError::Upstream(resp.status()));
    }

    let bytes = resp.bytes().await?.to_vec();

    // best-effort cache save
    let _ = save_cache(&bytes).await;

    Ok(as_json_response(bytes))
}
