/*use reqwest::Client;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(serde::Deserialize)]
struct MeResponse {
    data: MeData,
}

#[derive(serde::Deserialize)]
struct MeData {
    user_id: Uuid,
}

pub async fn resolve_user_id(
    users_ms_url: &str,
    auth_header: &str,
) -> Result<Uuid, ApiError> {

    let res = Client::new()
        .get(format!("{users_ms_url}/me"))
        .header("authorization", auth_header)
        .send()
        .await
        .map_err(|_| ApiError::upstream_unavailable("users"))?;

    if !res.status().is_success() {
        return Err(ApiError::upstream_invalid_response("users"));
    }

    let body: MeResponse = res
        .json()
        .await
        .map_err(|_| ApiError::upstream_invalid_response("users"))?;

    Ok(body.data.user_id)
}*/

/*
use reqwest::Client;
use uuid::Uuid;

use axum::http::HeaderMap;

use crate::error::ApiError;

#[derive(serde::Deserialize)]
struct MeResponse {
    data: MeData,
}

#[derive(serde::Deserialize)]
struct MeData {
    user_id: Uuid,
}

pub async fn resolve_user_id(
    users_ms_url: &str,
    headers: &HeaderMap,
) -> Result<Uuid, ApiError> {
    let url = format!("{users_ms_url}/me");
    println!("GATEWAY → USERS-MS GET {}", url);

    let client = Client::new();
    let mut req = client.get(url);

    for header in [
        "x-altair-keycloak-id",
        "x-altair-name",
        "x-altair-email",
        "x-altair-roles",
    ] {
        if let Some(value) = headers.get(header) {
            if let Ok(v) = value.to_str() {
                req = req.header(header, v);
            }
        }
    }

    let res = req
        .send()
        .await
        .map_err(|_| ApiError::upstream_unavailable("users"))?;

    if !res.status().is_success() {
        return Err(ApiError::upstream_invalid_response("users"));
    }

    let body: MeResponse = res
        .json()
        .await
        .map_err(|_| ApiError::upstream_invalid_response("users"))?;

    Ok(body.data.user_id)
}*/

use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::ApiError;

pub async fn resolve_user_id(
    users_ms_url: &str,
    keycloak_id: &str,
    email: &str,
    name: &str,
    roles: &str,
) -> Result<Uuid, ApiError> {
    let res = Client::new()
        .get(format!("{users_ms_url}/me"))
        .header("x-altair-keycloak-id", keycloak_id)
        .header("x-altair-email", email)
        .header("x-altair-name", name)
        .header("x-altair-roles", roles)
        .send()
        .await
        .map_err(|_| ApiError::upstream_unavailable("users"))?;

    if !res.status().is_success() {
        return Err(ApiError::upstream_invalid_response("users"));
    }

    let json: Value = res
        .json()
        .await
        .map_err(|_| ApiError::upstream_invalid_response("users"))?;

    let user_id = json
        .get("data")
        .and_then(|d| d.get("user_id"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| ApiError::upstream_invalid_response("users"))?;

    Uuid::parse_str(user_id).map_err(|_| ApiError::upstream_invalid_response("users"))
}
