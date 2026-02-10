#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Creator,
    Learner,
}

impl Role {
    pub fn from_str(role: &str) -> Option<Self> {
        match role {
            "learner" => Some(Role::Learner),
            "creator" => Some(Role::Creator),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}
