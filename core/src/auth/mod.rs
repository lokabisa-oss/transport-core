#[derive(Debug, Clone)]
pub enum AuthDecision {
    RefreshAndRetry,
    Fail,
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub refresh_in_progress: bool,
    pub refresh_attempted: bool,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            refresh_in_progress: false,
            refresh_attempted: false,
        }
    }
}
