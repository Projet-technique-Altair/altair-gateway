use crate::services::{users_api::UsersApi, labs_api::LabsApi, sessions_api::SessionsApi};

#[derive(Clone)]
pub struct AppState {
    pub users: UsersApi,
    pub labs: LabsApi,
    pub sessions: SessionsApi,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: UsersApi::new(std::env::var("USERS_MS_URL").unwrap()),
            labs: LabsApi::new(std::env::var("LABS_MS_URL").unwrap()),
            sessions: SessionsApi::new(std::env::var("SESSIONS_MS_URL").unwrap()),
        }
    }
}
