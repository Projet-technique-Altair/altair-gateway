use crate::services::{users_api::UsersApi, labs_api::LabsApi, sessions_api::SessionsApi};

#[derive(Clone)]
pub struct AppState {
    pub users: UsersApi,
    pub labs: LabsApi,
    pub sessions: SessionsApi,
}

impl AppState {
    pub fn new() -> Self {
        let users_url =
            std::env::var("USERS_MS_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());
        let labs_url =
            std::env::var("LABS_MS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        let sessions_url =
            std::env::var("SESSIONS_MS_URL").unwrap_or_else(|_| "http://localhost:3003".to_string());

        Self {
            users: UsersApi::new(users_url),
            labs: LabsApi::new(labs_url),
            sessions: SessionsApi::new(sessions_url),
        }
    }
}

