use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::{
    error::ApiError,
    security::roles::Role,
};

pub async fn rbac_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    

    // =========================
    // Public endpoints
    // =========================
    let path = req.uri().path();
    if path == "/health" {
        return Ok(next.run(req).await);
    }

    // =========================
    // Extract service name
    // =========================
    // /labs/anything/...  -> labs
    // /sessions/foo/bar   -> sessions
    let service = path
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or("");

    // =========================
    // Roles injected by JWT
    // =========================
    let roles = req
        .extensions()
        .get::<Vec<Role>>()
        .cloned()
        .unwrap_or_default();

    
    println!("RBAC path = {}", req.uri().path());
    println!("RBAC roles = {:?}", roles);

    // =========================
    // Admin bypass
    // =========================
    if roles.contains(&Role::Admin) {
        return Ok(next.run(req).await);
    }

    let is_learner = roles.contains(&Role::Learner) || roles.contains(&Role::Creator);
    let is_creator = roles.contains(&Role::Creator);

    let method = req.method().as_str();


    // =========================
    // Self identity endpoint
    // =========================
    /*if path == "/users/me" {
        return Ok(next.run(req).await);
    }*/


    // =========================
    // RBAC RULES (SERVICE-LEVEL)
    // =========================
    let authorized = match (method, service) {

        // =====================
        // READ ACCESS
        // =====================
        ("GET", "labs") => is_learner,
        ("GET", "sessions") => is_learner,
        ("GET", "users") => is_learner,
        ("GET", "starpaths") => is_learner,
        ("GET", "groups") => is_learner,

        // =====================
        // WRITE ACCESS
        // =====================
        ("POST", "labs") => is_creator,
        ("PUT", "labs") => is_creator,
        ("DELETE", "labs") => is_creator,

        ("POST", "sessions") => is_creator,
        ("PUT", "sessions") => is_creator,
        ("DELETE", "sessions") => is_creator,

        ("POST", "users") => is_creator,
        ("PUT", "users") => is_creator,
        ("DELETE", "users") => is_creator,

        ("POST", "starpaths") => is_creator,
        ("PUT", "starpaths") => is_creator,
        ("DELETE", "starpaths") => is_creator,

        ("POST", "groups") => is_creator,
        ("PUT", "groups") => is_creator,
        ("DELETE", "groups") => is_creator,

        // =====================
        // DEFAULT DENY
        // =====================
        _ => false,
    };

    if !authorized {
        return Err(ApiError::forbidden(
            "You do not have permission to access this resource",
        ));
    }

    Ok(next.run(req).await)
}

