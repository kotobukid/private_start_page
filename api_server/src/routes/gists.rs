use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct GistsResponse {
    pub gists: Vec<BookmarkList>,
}

#[derive(Serialize)] // Responseとして返すためにSerializeを追加
pub struct BookmarkList {
    pub title: String,
    pub url: String,
    pub html_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gist {
    pub files: HashMap<String, GistFile>,
    pub description: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GistFile {
    pub filename: String,
    #[serde(rename = "type")] // "type" は予約語なのでリネーム
    pub file_type: String,
    pub language: Option<String>, // 言語判定不能な場合はnullがありえる
    pub raw_url: String,
    pub size: i32,
}

pub async fn get_gists(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let url = "https://api.github.com/gists";

    let resp = state
        .http
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", state.github_token))
        .header(USER_AGENT, "private_start_page/0.1")
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(ApiError::Upstream(resp.status()));
    }

    let api_gists: Vec<Gist> = resp.json().await?;

    let gists: Vec<BookmarkList> = api_gists
        .into_iter()
        .filter_map(|gist| {
            let desc = gist.description.as_deref().unwrap_or("");
            if !desc.contains("bookmark") && !desc.contains("ブックマーク") {
                return None;
            }

            let target_file = gist
                .files
                .values()
                .find(|f| f.file_type == "application/json")?;

            Some(BookmarkList {
                title: target_file.filename.clone(),
                url: target_file.raw_url.clone(),
                html_url: gist.html_url.clone(),
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(GistsResponse { gists })))
}
