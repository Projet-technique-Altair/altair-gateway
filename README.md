# Altaïr API Gateway

> **Unified entry point for authentication, authorization, and intelligent request routing**
> 

[![Cloud Run](https://img.shields.io/badge/deploy-Cloud%20Run-blue)](https://cloud.google.com/run)

[![Rust](https://img.shields.io/badge/rust-nightly-orange)](https://www.rust-lang.org)

[![Keycloak](https://img.shields.io/badge/auth-Keycloak-blue)](https://www.keycloak.org)

---

## Description

The **Altaïr API Gateway** is the single public-facing entry point for all backend services in the Altaïr platform. It handles JWT authentication, role-based access control, internal identity resolution, and intelligent routing to downstream microservices.

This service acts as the **security boundary**: it validates Keycloak JWT tokens and injects verified identity headers that downstream microservices trust without performing their own authentication.

**Key capabilities:**

- Validate Keycloak JWT tokens (RS256 with JWKS)
- Enforce role-based access control (Admin, Creator, Learner)
- Resolve Keycloak IDs to internal user UUIDs (with caching)
- Route requests to appropriate microservices
- Inject trusted identity headers for downstream services
- Provide unified error responses

---

## ⚠️ Security Notice

**This service is the trust boundary for the entire platform.**

- ✅ **Only service that validates JWTs** – downstream services trust injected headers
- ✅ **Anti-spoofing protection** – strips incoming `x-altair-*` headers before injection
- ⚠️ **In-memory cache** – user ID cache is per-instance (not shared across replicas)
- ✅ **JWKS cache enabled** – TTL + `cache-control` support + forced refresh on unknown `kid`

**Deployment requirement:** Must be the ONLY publicly accessible service. All microservices should be internal-only.

---

## Architecture

```
┌─────────────┐                  ┌──────────────────┐
│  Frontend   │                  │   API Gateway    │
│  (Browser)  │─────JWT─────────▶│   (Cloud Run)    │
│             │                  │     :3000        │
└─────────────┘                  └────────┬─────────┘
                                          │
                        ┌─────────────────┼─────────────────┐
                        │                 │                 │
                        ▼                 ▼                 ▼
                 ┌─────────────┐   ┌─────────────┐  ┌─────────────┐
                 │  Users MS   │   │  Labs MS    │  │ Sessions MS │
                 │   (3001)    │   │   (3002)    │  │   (3003)    │
                 └─────────────┘   └─────────────┘  └─────────────┘
                        │                 │                 │
                        ▼                 ▼                 ▼
                 ┌─────────────┐   ┌─────────────┐  ┌─────────────┐
                 │ PostgreSQL  │   │ PostgreSQL  │  │ PostgreSQL  │
                 │   (Users)   │   │   (Labs)    │  │  (Sessions) │
                 └─────────────┘   └─────────────┘  └─────────────┘
```

### Request Flow

1. **Frontend** sends request with `Authorization: Bearer <JWT>`
2. **Gateway** validates JWT against Keycloak JWKS
3. **Gateway** extracts roles and identity claims
4. **Gateway** checks RBAC rules for service access
5. **Gateway** resolves Keycloak ID → internal user UUID (cached)
6. **Gateway** strips incoming `x-altair-*` headers (anti-spoofing)
7. **Gateway** injects trusted headers (`x-altair-user-id`, etc.)
8. **Gateway** proxies request to appropriate microservice
9. **Microservice** trusts headers without JWT validation
10. **Gateway** returns response to frontend

---

## Tech Stack

| Component | Technology | Purpose |
| --- | --- | --- |
| **Language** | Rust (nightly) | High-performance async runtime |
| **HTTP Framework** | Axum | HTTP routing and middleware |
| **Async Runtime** | Tokio | Async I/O and concurrency |
| **JWT Validation** | jsonwebtoken | RS256 signature verification |
| **HTTP Client** | reqwest | Upstream microservice calls |
| **Caching** | DashMap | Concurrent user ID cache |
| **Middleware** | tower-http | CORS configuration |
| **CI/CD** | GitHub Actions | fmt, clippy, tests, build |
| **Deployment** | Google Cloud Run | Serverless auto-scaling |

---

## Requirements

### Development

- **Rust** nightly toolchain
- **Running Keycloak** instance (via `altair-infra`)
- **Running microservices** (users-ms at minimum)

### Production (Cloud Run)

- **Keycloak** realm with JWKS endpoint
- **Internal URLs** for all backend microservices
- **Environment Variables** (see Configuration section)

### Environment Variables

#### Local Development

```bash
# Keycloak Configuration (required)
KEYCLOAK_ISSUER=http://localhost:8080/realms/altair
KEYCLOAK_JWKS_URL=http://localhost:8080/realms/altair/protocol/openid-connect/certs
JWKS_TTL_SECONDS=300
JWKS_STALE_IF_ERROR_SECONDS=120
JWKS_CACHE_MIN_TTL_SECONDS=30
JWKS_CACHE_MAX_TTL_SECONDS=3600

# Microservice URLs (defaults available)
USERS_MS_URL=http://localhost:3001
LABS_MS_URL=http://localhost:3002
SESSIONS_MS_URL=http://localhost:3003
STARPATH_MS_URL=http://localhost:3005
GROUPS_MS_URL=http://localhost:3006

# Server Configuration
PORT=3000                                 # Server port (default: 3000)
RUST_LOG=info                             # Log level filter

# CORS Configuration (CSV)
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
ALLOWED_METHODS=GET,POST,PUT,PATCH,DELETE,OPTIONS
ALLOWED_HEADERS=authorization,content-type
```

#### Production (Cloud Run)

```bash
# Keycloak Configuration (required)
KEYCLOAK_ISSUER=https://auth.altair.io/realms/altair
KEYCLOAK_JWKS_URL=https://auth.altair.io/realms/altair/protocol/openid-connect/certs
JWKS_TTL_SECONDS=300
JWKS_STALE_IF_ERROR_SECONDS=120
JWKS_CACHE_MIN_TTL_SECONDS=30
JWKS_CACHE_MAX_TTL_SECONDS=3600

# Microservice URLs (internal Cloud Run services)
USERS_MS_URL=https://users-ms-xxx.run.app
LABS_MS_URL=https://labs-ms-xxx.run.app
SESSIONS_MS_URL=https://sessions-ms-xxx.run.app
STARPATH_MS_URL=https://starpath-ms-xxx.run.app
GROUPS_MS_URL=https://groups-ms-xxx.run.app

# Server Configuration
PORT=3000
RUST_LOG=info

# CORS Configuration (explicit frontend domains)
ALLOWED_ORIGINS=https://app.altair.io
ALLOWED_METHODS=GET,POST,PUT,PATCH,DELETE,OPTIONS
ALLOWED_HEADERS=authorization,content-type
```

CORS env format rules:
- `ALLOWED_ORIGINS`: comma-separated origins
- `ALLOWED_METHODS`: comma-separated HTTP methods
- `ALLOWED_HEADERS`: comma-separated request header names (lowercase preferred)
- Missing/invalid env values fallback to safe defaults in `src/main.rs`.

JWKS cache env format rules:
- `JWKS_TTL_SECONDS`: default TTL when `cache-control` does not provide `max-age`.
- `JWKS_STALE_IF_ERROR_SECONDS`: grace window to use stale JWKS if refresh fails.
- `JWKS_CACHE_MIN_TTL_SECONDS`: minimum accepted TTL.
- `JWKS_CACHE_MAX_TTL_SECONDS`: maximum accepted TTL.

JWKS runtime logs (terminal):
- `[JWKS] cache hit` → valid JWKS found in memory, no network call to Keycloak.
- `[JWKS] cache miss/expired` → no valid JWKS in cache, refresh is required.
- `[JWKS] refresh success (ttl=Ns)` → refresh succeeded, cache renewed for `N` seconds.
- `[JWKS] unknown kid, forcing refresh: <kid>` → token key id not found in cache, immediate refresh forced.
- `[JWKS] refresh failed, using stale cache` → refresh failed but stale window is still active.
- `[JWKS] refresh failed, no stale cache available` → refresh failed and no fallback cache remains; request will fail.

---

## Installation

### Local Development

```bash
# 1. Ensure infrastructure is running
cd ../altair-infra
docker compose up -d

# 2. Ensure users-ms is running (minimum requirement)
cd ../altair-users-ms
cargo run

# 3. Run the gateway
cd ../altair-gateway
cargo run

# 4. Test the health endpoint
curl http://localhost:3000/health
```

### Building Docker Image

```bash
# Build the container
docker build -t altair-gateway:latest .

# Run locally
docker run --rm -it \
  -p 3000:3000 \
  --network altair-infra_default \
  --env-file .env \
  altair-gateway:latest
```

### Deploying to Cloud Run

```bash
# 1. Build and push to Artifact Registry
gcloud builds submit --tag europe-west9-docker.pkg.dev/PROJECT/altair/gateway

# 2. Deploy to Cloud Run (public access)
gcloud run deploy altair-gateway \
  --image europe-west9-docker.pkg.dev/PROJECT/altair/gateway \
  --region europe-west9 \
  --platform managed \
  --allow-unauthenticated \
  --set-env-vars KEYCLOAK_ISSUER=https://auth.altair.io/realms/altair \
  --set-env-vars USERS_MS_URL=https://users-ms-xxx.run.app \
  --set-env-vars LABS_MS_URL=https://labs-ms-xxx.run.app
```

---

## Usage

### API Endpoints

#### **GET /health**

Health check for liveness/readiness probes.

**Response:**

```json
{
  "status": "ok"
}
```

**Note:** Does NOT require authentication.

---

#### **Catch-All Protected Routes: /:service/*rest**

All other routes are protected and proxied to backend services.

**Authentication:** Requires valid JWT in `Authorization: Bearer <token>` header.

**Route Pattern:**

```
GET /users/me           → {USERS_MS_URL}/me
POST /sessions/start    → {SESSIONS_MS_URL}/start
GET /labs/123           → {LABS_MS_URL}/123
```

**Injected Headers (downstream services):**

- `x-altair-user-id` – Internal user UUID
- `x-altair-keycloak-id` – Keycloak user ID (sub claim)
- `x-altair-name` – User display name
- `x-altair-email` – User email address
- `x-altair-roles` – Comma-separated role list (e.g., `learner,creator`)

---

## JWT Authentication

### Token Validation Process

1. **Extract Token**
    - Read `Authorization: Bearer <token>` header
    - Decode JWT header to extract `kid` (key ID)
2. **Resolve JWKS (cached)**
    - Use in-memory JWKS cache if valid (TTL / `cache-control`)
    - On cache miss/expiry, refresh from `KEYCLOAK_JWKS_URL`
    - If `kid` is unknown, force one immediate refresh, then re-check key
3. **Verify Signature**
    - Algorithm: RS256
    - Issuer: Must match `KEYCLOAK_ISSUER`
    - Expiration: Checked and enforced
    - Audience: **Currently disabled** for MVP
4. **Extract Claims**
    - `sub` → Keycloak user ID
    - `name` or `preferred_username` → Display name
    - `email` → Email address
    - `realm_access.roles` → Realm roles
    - `resource_access["frontend"].roles` → Client roles

### Role Mapping

Roles from JWT claims are mapped to internal enum:

| JWT Role String | Internal Enum | Access Level |
| --- | --- | --- |
| `admin` | `Role::Admin` | Full access to all services |
| `creator` | `Role::Creator` | Read + write access |
| `learner` | `Role::Learner` | Read-only access |

---

## Role-Based Access Control

### Service-Level RBAC

Access control is enforced at the **service level**, not the route level.

**Authorization Matrix:**

| Role | Services | GET (Read) | POST/PUT/DELETE (Write) |
| --- | --- | --- | --- |
| **Admin** | All | ✅ | ✅ |
| **Creator** | users, labs, sessions, starpaths, groups | ✅ | ✅ |
| **Learner** | users, labs, sessions, starpaths, groups | ✅ | ❌ |

**Examples:**

- ✅ Learner can `GET /labs/123`
- ❌ Learner cannot `POST /labs` (403 Forbidden)
- ✅ Creator can `POST /labs` (create new lab)
- ✅ Admin can access any service with any method

**Note:** Fine-grained authorization (e.g., "can this user edit THIS specific lab?") is handled by individual microservices, not the gateway.

---

## Identity Resolution

The gateway resolves Keycloak identities to internal user UUIDs by calling the Users microservice.

### Resolution Flow

1. **Extract Keycloak ID** from JWT `sub` claim
2. **Check cache** (`DashMap<String, Uuid>`)
    - Cache key: Keycloak ID
    - Cache value: Internal user UUID
3. **On cache miss:**
    - Call `GET {USERS_MS_URL}/me` with headers:
        - `x-altair-keycloak-id`
        - `x-altair-name`
        - `x-altair-email`
        - `x-altair-roles`
    - Parse `data.user_id` from response
    - Store in cache
4. **Inject `x-altair-user-id`** header for downstream

**Cache characteristics:**

- **Per-instance** (not shared across Cloud Run replicas)
- **In-memory** (no persistence)
- **Concurrent** (uses `DashMap` for lock-free reads)
- **No expiration** (entries live forever)

**Cache miss handling:**

- First request from a user: cache miss, calls users-ms
- Subsequent requests: cache hit, no users-ms call
- New replica instance: cold cache, misses until warmed

---

## Reverse Proxy

The gateway proxies validated requests to backend microservices.

### URL Construction

**Pattern:** `upstream_url = base_url + "/" + rest`

**Service prefix is stripped:**

- Incoming: `/users/me`
- Service: `users`, Rest: `me`
- Upstream: `{USERS_MS_URL}/me` (not `/users/me`)

**Service Registry (from environment):**

| Service Name | Environment Variable | Default Local URL |
| --- | --- | --- |
| `users` | `USERS_MS_URL` | http://localhost:3001 |
| `labs` | `LABS_MS_URL` | http://localhost:3002 |
| `sessions` | `SESSIONS_MS_URL` | http://localhost:3003 |
| `starpaths` | `STARPATH_MS_URL` | http://localhost:3005 |
| `groups` | `GROUPS_MS_URL` | http://localhost:3006 |

### Request Forwarding

**Headers forwarded:**

- Most incoming headers
- **Skipped:** `Host`, `Content-Length`, `Connection`, `Origin`, `Referer`

**Body handling:**

- Entire request body buffered in memory
- Sent to upstream service

### Response Forwarding

**Forwarded:**

- HTTP status code
- Response body
- `Content-Type`
- `Set-Cookie`
- `Location`
- Cache headers: `Cache-Control`, `ETag`, `Last-Modified`, `Expires`, `Vary`

**Not forwarded:**

- All non-allowlisted headers (intentional strict policy)

---

## Project Structure

```
altair-gateway/
├── Cargo.toml                    # Rust dependencies
├── Cargo.lock                    # Locked dependency versions
├── Dockerfile                    # Multi-stage build
├── README.md                     # This file
├── .env                          # Environment variables (NOT committed)
├── .gitignore
├── requests.http                 # HTTP test scenarios
└── src/
    ├── main.rs                  # Server bootstrap, CORS, routes
    ├── state.rs                 # AppState (service registry + cache)
    ├── error.rs                 # ApiError type
    ├── utils.rs                 # Utilities
    ├── routes/
    │   ├── mod.rs              # Router configuration
    │   └── health.rs           # Health check endpoint
    ├── middleware/
    │   ├── jwt.rs              # JWT validation & identity resolution
    │   └── rbac.rs             # Role-based access control
    ├── services/
    │   ├── proxy.rs            # Reverse proxy logic
    │   ├── users_client.rs     # Users microservice client
    │   └── common.rs           # HTTP client utilities
    ├── security/
    │   └── roles.rs            # Role enum definition
    └── tests/
        └── *.rs                # Unit tests (WIP)
```

---

## Deployment (Cloud Run)

The Gateway is deployed to **Google Cloud Run** as the single public-facing entry point.

### Container Configuration

- Listens on port defined by `PORT` environment variable (default: `3000`)
- Multi-stage Docker build (Rust builder → Debian slim runtime)
- Stateless except for in-memory user cache

### Network Architecture

**Public Access:**

- Gateway is the **ONLY publicly accessible service**
- All backend microservices are internal (Cloud Run internal-only)

**Service-to-Service Communication:**

- Gateway → Microservices via internal Cloud Run URLs
- No public IP exposure for backend services
- VPC connector may be required for non-Cloud Run services

### Service Account Permissions

The Cloud Run service account requires:

- **Cloud Run Invoker** role for calling internal Cloud Run services
- Network access to internal services (VPC connector if needed)
- No special GCP API permissions required

### Scaling Behavior

- **Min instances:** 0 (scales to zero when idle)
- **Max instances:** Configurable (default: 100)
- **Cold start time:** ~2-5 seconds (Rust fast startup)
- **Concurrency:** 80 requests per instance (default)

**Cache implications:**

- Each instance has its own user ID cache
- Cache misses on scale-up are handled gracefully
- No cache synchronization across instances

---

## Known Issues & Limitations

### 🔴 Critical Issues

- **In-memory JWKS cache** – cache is per-instance (not shared across replicas)
- **Health endpoint bypass mismatch** – JWT bypasses `/*/health`, RBAC only bypasses `/health`

### 🟡 Operational Limitations

- **Non-streaming proxy** – Request/response bodies fully buffered in memory
- **Limited header forwarding** – strict allowlist only (not full header passthrough)
- **Per-instance cache** – User ID cache not shared across replicas
- **No cache expiration** – Entries live forever (memory leak risk on long-running instances)

### 🟡 Security Concerns

- **Permissive CORS** – Allows any origin (acceptable for dev, dangerous for production)
- **Audience validation disabled** – JWT `aud` claim not validated (acceptable for MVP)
- **Plaintext logging** – Upstream response bodies logged (may contain sensitive data)

### 🟡 Missing Features

- **No metrics collection** – Cannot monitor cache hit rate, upstream latency
- **No circuit breakers** – Failed upstream services can cascade failures
- **No request retry logic** – Transient failures are not retried
- **No rate limiting** – Vulnerable to abuse

---

## Troubleshooting

### Gateway Won't Start

**Symptom:** Service fails to initialize.

**Solution:**

```bash
# Check Keycloak is reachable
curl $KEYCLOAK_ISSUER/.well-known/openid-configuration

# Verify environment variables
echo $KEYCLOAK_ISSUER
echo $USERS_MS_URL
```

### JWT Validation Fails

**Symptom:** All requests return `401 Unauthorized`.

**Possible causes:**

- Keycloak JWKS endpoint unreachable
- Issuer mismatch between token and configuration
- Token expired

**Debug:**

```bash
# Test JWT decoding
jwt decode $TOKEN

# Check issuer matches
echo $KEYCLOAK_ISSUER
jwt decode $TOKEN | jq .iss
```

### 403 Forbidden on Health Checks

**Symptom:** `GET /users/health` returns `403 Forbidden`.

**Cause:** RBAC middleware only bypasses exact `/health`, not `/*/health`.

**Workaround:** Call health checks directly on microservices (not via gateway).

### Slow Response Times

**Symptom:** High latency on all requests.

**Cause:** cache miss/expiry, upstream latency, or downstream service latency.

**Solution:** verify `JWKS_*` settings and upstream health (`KEYCLOAK_JWKS_URL`).

---

## TODO / Roadmap

### High Priority (MVP → Production)

- [x]  **Implement JWKS caching** (TTL + cache-control + forced refresh on unknown kid)
- [x]  **Fix service naming** (aligned on `starpaths` across registry and RBAC)
- [ ]  **Fix health endpoint bypass** (consistent JWT + RBAC)
- [x]  **Restrict CORS** (env-driven strict origins, methods, headers)

### Medium Priority (Production Hardening)

- [ ]  **Add streaming proxy** (avoid buffering large bodies)
- [x]  **Forward critical response headers** (`Set-Cookie`, `Location`, cache headers)
- [ ]  **Implement circuit breakers** (handle upstream failures gracefully)
- [ ]  **Add request retry logic** (with exponential backoff)
- [ ]  **Enable audience validation** (validate JWT `aud` claim)

### Low Priority (Future Enhancements)

- [ ]  **Shared cache** (Redis for user ID cache across instances)
- [ ]  **Rate limiting** (per-user quotas)
- [ ]  **Metrics collection** (Prometheus/Grafana integration)
- [ ]  **Distributed tracing** (OpenTelemetry)
- [ ]  **GraphQL gateway** (unified API schema)

---

## Project Status

**✅ Current Status: MVP (Minimum Viable Product)**

This gateway is **functional for MVP deployment** with core authentication, authorization, and routing capabilities operational. Performance optimizations and operational improvements remain before production-ready status.

**Known limitations to address for production:**

1. Service naming consistency fix
2. Health endpoint bypass consistency
3. Streaming proxy + full response header forwarding
4. Streaming proxy implementation
5. Comprehensive header forwarding
6. Circuit breaker patterns

**Maintainers:** Altaïr Platform Team

---

## Notes

- **Trust boundary** – Only service that validates JWTs
- **Stateless except cache** – User ID cache is per-instance
- **Anti-spoofing protection** – Strips incoming `x-altair-*` headers
- **Service prefix stripping** – `/users/me` becomes `/me` to upstream
- **Single public endpoint** – All backend services should be internal-only

---

## License

Internal Altaïr Platform Service – Not licensed for external use.
