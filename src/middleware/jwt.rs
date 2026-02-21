use axum::extract::State;
use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::error::ApiError;
use crate::security::jwks_cache::Jwk;
use crate::security::roles::Role;
use crate::services::users_client::resolve_user_id;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct RealmAccess {
    roles: Vec<String>,
}

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ResourceAccess {
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(rename = "exp")]
    _exp: usize,
    #[serde(rename = "iss")]
    _iss: String,
    #[serde(rename = "aud")]
    _aud: serde_json::Value,

    // 👇 AJOUT ICI
    realm_access: Option<RealmAccess>,

    // client roles
    resource_access: Option<HashMap<String, ResourceAccess>>,

    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
}

const HDR_ROLES: &str = "x-altair-roles";
const HDR_KEYCLOAK_ID: &str = "x-altair-keycloak-id";
const HDR_EMAIL: &str = "x-altair-email";
const HDR_NAME: &str = "x-altair-name";
const HDR_USER_ID: &str = "x-altair-user-id";

use std::collections::HashSet;

fn extract_roles(claims: &Claims) -> Vec<Role> {
    let mut set = HashSet::new();

    // 1️⃣ Realm roles
    if let Some(realm) = &claims.realm_access {
        for r in &realm.roles {
            if let Some(role) = Role::from_str(r) {
                set.insert(role);
            }
        }
    }

    // 2️⃣ Client roles (frontend)
    let client_id = "frontend";

    if let Some(resource_access) = &claims.resource_access {
        if let Some(client) = resource_access.get(client_id) {
            for r in &client.roles {
                if let Some(role) = Role::from_str(r) {
                    set.insert(role);
                }
            }
        }
    }

    set.into_iter().collect()
}

pub async fn jwt_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Public endpoint (no auth)
    let path = req.uri().path();
    if path == "/health" {
        return Ok(next.run(req).await);
    }

    let auth = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("Invalid Authorization header"))?;

    let header = decode_header(token).map_err(|_| ApiError::unauthorized("Invalid JWT header"))?;

    let kid = header
        .kid
        .as_ref()
        .ok_or_else(|| ApiError::unauthorized("Missing kid in JWT"))?
        .clone();

    let issuer = std::env::var("KEYCLOAK_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080/realms/altair".to_string());

    let jwks_url = std::env::var("KEYCLOAK_JWKS_URL")
        .unwrap_or_else(|_| format!("{issuer}/protocol/openid-connect/certs"));

    let jwk: Jwk = state.jwks_cache.get_jwk_for_kid(&jwks_url, &kid).await?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|_| ApiError::unauthorized("Invalid JWKS key"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.set_issuer(std::slice::from_ref(&issuer));
    validation.validate_aud = false; // MVP

    //let token_data = decode::<Claims>(token, &decoding_key, &validation)
    //.map_err(|_| ApiError::unauthorized("Invalid JWT"))?;

    println!("===== JWT DEBUG =====");
    println!("Expected issuer      : {}", issuer);
    println!("Token kid             : {:?}", header.kid);
    println!("Validation issuer set : {:?}", validation.iss);
    println!("Validation aud        : {:?}", validation.aud);

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        println!("JWT decode error: {:?}", e);
        ApiError::unauthorized("Invalid JWT")
    })?;

    let keycloak_id = token_data.claims.sub.clone();

    // =========================
    // Extract identity infos
    // =========================

    let roles = extract_roles(&token_data.claims);
    req.extensions_mut().insert(roles.clone());

    let roles_csv = roles
        .iter()
        .map(|r| format!("{r:?}").to_lowercase())
        .collect::<Vec<_>>()
        .join(",");

    let name = token_data
        .claims
        .name
        .clone()
        .or(token_data.claims.preferred_username.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let email = token_data
        .claims
        .email
        .clone()
        .unwrap_or_else(|| "unknown@altair.local".to_string());

    // =========================
    // Resolve internal user_id
    // =========================

    // Nettoyage anti-spoof
    req.headers_mut().remove(HDR_USER_ID);

    let user_id = if let Some(entry) = state.user_cache.get(&keycloak_id) {
        // 🟢 cache hit
        *entry
    } else {
        // 🔵 cache miss → call users-ms
        let users_ms_url = state
            .services
            .get("users")
            .ok_or_else(|| ApiError::upstream_unavailable("users"))?;
        //let resolved_user_id =
        //resolve_user_id(users_ms_url, auth_header).await?;
        let resolved_user_id =
            resolve_user_id(users_ms_url, &keycloak_id, &email, &name, &roles_csv).await?;

        state
            .user_cache
            .insert(keycloak_id.clone(), resolved_user_id);

        resolved_user_id
    };

    // Injection du user_id pour les autres MS
    req.headers_mut().insert(
        HeaderName::from_static(HDR_USER_ID),
        HeaderValue::from_str(&user_id.to_string())
            .map_err(|_| ApiError::unauthorized("Invalid user id"))?,
    );

    /*let roles = extract_roles(&token_data.claims);
    req.extensions_mut().insert(roles.clone());


    let name = token_data.claims.name
        .clone()
        .or(token_data.claims.preferred_username.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let email = token_data.claims.email
        .clone()
        .unwrap_or_else(|| "unknown@altair.local".to_string());*/

    // Anti-spoof: nettoyage
    req.headers_mut().remove(HDR_KEYCLOAK_ID);
    req.headers_mut().remove(HDR_EMAIL);
    req.headers_mut().remove(HDR_NAME);
    req.headers_mut().remove(HDR_ROLES);

    // Injection headers vers MS
    req.headers_mut().insert(
        HeaderName::from_static(HDR_KEYCLOAK_ID),
        HeaderValue::from_str(&keycloak_id)
            .map_err(|_| ApiError::unauthorized("Invalid keycloak id"))?,
    );

    req.headers_mut().insert(
        HeaderName::from_static(HDR_NAME),
        HeaderValue::from_str(&name).map_err(|_| ApiError::unauthorized("Invalid name"))?,
    );

    req.headers_mut().insert(
        HeaderName::from_static(HDR_EMAIL),
        HeaderValue::from_str(&email).map_err(|_| ApiError::unauthorized("Invalid email"))?,
    );

    let roles_csv = roles
        .iter()
        .map(|r| format!("{r:?}").to_lowercase())
        .collect::<Vec<_>>()
        .join(",");

    req.headers_mut().insert(
        HeaderName::from_static(HDR_ROLES),
        HeaderValue::from_str(&roles_csv).map_err(|_| ApiError::unauthorized("Invalid roles"))?,
    );

    Ok(next.run(req).await)
}
