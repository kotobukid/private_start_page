use std::net::SocketAddr;

use axum::response::Html;
use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;

mod error;
mod routes;
mod state;

use crate::routes::{bookmarks::serve_bookmarks, gists::get_gists};
use crate::state::AppState;

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

    let state = AppState { github_token: token, http };

    let app = Router::new()
        .route("/", get(Html("<a href=\"/gists.json\">gists</a>")))
        .route("/api/gists.json", get(get_gists))
        .route("/api/bookmarks.json", get(serve_bookmarks))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("API server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
