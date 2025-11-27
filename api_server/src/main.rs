use std::collections::HashMap;
use std::net::SocketAddr;

use axum::response::Html;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use dotenvy::dotenv;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone)]
struct AppState {
    github_token: String,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct GistsResponse {
    gists: Vec<BookmarkList>, // 型を Gist から Bookmark に変更
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[allow(dead_code)]
    #[error("環境変数 GITHUB_ACCESS_TOKEN が見つかりません")]
    MissingToken,
    #[error("GitHub API リクエストに失敗しました: {0}")]
    Request(#[from] reqwest::Error),
    #[error("GitHub API からエラー応答が返されました: {0}")]
    Upstream(StatusCode),
    #[allow(dead_code)]
    #[error("内部エラーが発生しました")]
    Internal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gist {
    // pub url: String,
    // pub forks_url: String,
    // pub commits_url: String,
    // pub id: String,
    // pub node_id: String,
    // pub git_pull_url: String,
    // pub git_push_url: String,
    // pub html_url: String,
    pub files: HashMap<String, GistFile>, // ここがポイント: キーがファイル名のマップ
    // pub public: bool,
    // pub created_at: chrono::DateTime<chrono::Utc>,
    // pub updated_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    // pub comments: i32,
    // pub user: Option<serde_json::Value>, // null または 詳細不明なオブジェクト
    // pub comments_url: String,
    // pub owner: Option<GistOwner>, // ユーザ情報
    // pub truncated: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GistOwner {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub site_admin: bool,
    #[serde(rename = "type")]
    pub user_type: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::MissingToken => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Request(err) => {
                // GitHubからのエラー内容をそのままは返さず、要約のみ
                let code = err.status().unwrap_or(StatusCode::BAD_GATEWAY);
                (code, format!("GitHub API エラー: {}", code))
            }
            ApiError::Upstream(code) => (code, format!("GitHub API エラー: {}", code)),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, msg).into_response()
    }
}

#[tokio::main]
async fn main() {
    // .env の読み込み（存在しない場合は無視）
    let _ = dotenv();

    // 必須: GITHUB_ACCESS_TOKEN
    let token = match env::var("GITHUB_ACCESS_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => {
            eprintln!("エラー: 環境変数 GITHUB_ACCESS_TOKEN が設定されていません");
            // サーバを起動しても意味がないので終了
            std::process::exit(1);
        }
    };

    let http = reqwest::Client::builder()
        .user_agent("private_start_page/0.1 (+https://example.local)")
        .build()
        .expect("reqwest client");

    let state = AppState {
        github_token: token,
        http,
    };

    let app = Router::new()
        .route("/", get(Html("<a href=\"/gists.json\">gists</a>")))
        .route("/api/gists.json", get(get_gists))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("API server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)] // Responseとして返すためにSerializeを追加
struct BookmarkList {
    title: String,
    url: String,
}

async fn get_gists(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // GitHub API: GET /gists （認証済みユーザの公開/秘密含む）
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
        .into_iter() // 所有権ごとイテレータにする（文字列のcloneを減らすため）
        .filter_map(|gist| {
            // Descriptionのチェック
            let desc = gist.description.as_deref().unwrap_or("");
            if !desc.contains("bookmark") && !desc.contains("ブックマーク") {
                return None;
            }

            // Filesの中から "application/json" を探す (最初に見つかった1つだけ)
            let target_file = gist
                .files
                .values()
                .find(|f| f.file_type == "application/json")?;

            // Bookmark構造体に変換して返す
            Some(BookmarkList {
                title: target_file.filename.clone(),
                url: target_file.raw_url.clone(),
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(GistsResponse { gists })))
}
