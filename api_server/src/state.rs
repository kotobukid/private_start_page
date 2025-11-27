#[derive(Clone)]
pub struct AppState {
    pub github_token: String,
    pub http: reqwest::Client,
}
