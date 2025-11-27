use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[allow(dead_code)]
    #[error("環境変数 GITHUB_ACCESS_TOKEN が見つかりません")]
    MissingToken,
    #[error("HTTP リクエストに失敗しました: {0}")]
    Request(#[from] reqwest::Error),
    #[error("上流からエラー応答が返されました: {0}")]
    Upstream(StatusCode),
    #[allow(dead_code)]
    #[error("内部エラーが発生しました")]
    Internal,
    #[error("キャッシュ I/O エラー: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::MissingToken => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Request(err) => {
                let code = err.status().unwrap_or(StatusCode::BAD_GATEWAY);
                (code, format!("上流 HTTP エラー: {}", code))
            }
            ApiError::Upstream(code) => (code, format!("上流エラー: {}", code)),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "キャッシュ I/O エラー".into()),
        };
        (status, msg).into_response()
    }
}
