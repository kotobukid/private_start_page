use std::net::SocketAddr;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use std::env;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

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

    let state = AppState {
        github_token: token,
        http,
    };

    // 静的ファイル (front のビルド成果物) を配信
    // api_server クレート直下の `public/` をルートにする
    let public_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public");
    let static_files = ServeDir::new(&public_dir)
        // SPA ルーティング用フォールバック: 404 は index.html を返す
        .not_found_service(ServeFile::new(public_dir.join("index.html")));

    // API ルータを /api にネスト
    let api = Router::new()
        .route("/gists.json", get(get_gists))
        .route("/bookmarks.json", get(serve_bookmarks));

    let app = Router::new()
        .nest("/api", api)
        // Axum 0.8 ではルートへの nest は不可。未マッチは静的配信へフォールバック
        .fallback_service(static_files)
        .layer(CompressionLayer::new())
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("API server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
