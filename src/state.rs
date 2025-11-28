use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct AppState {
    pub labs: Arc<Mutex<Vec<Lab>>>,
    pub sessions: Arc<Mutex<Vec<LabSession>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            labs: Arc::new(Mutex::new(vec![
                Lab {
                    lab_id: "linux-basics".into(),
                    name: "Linux Basics".into(),
                    description: "Introduction to Linux".into(),
                    difficulty: 1,
                }
            ])),
            sessions: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lab {
    pub lab_id: String,
    pub name: String,
    pub description: String,
    pub difficulty: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LabSession {
    pub session_id: String,
    pub user_id: String,
    pub lab_id: String,
    pub container_id: String,
    pub status: String,
    pub webshell_url: String,
}
