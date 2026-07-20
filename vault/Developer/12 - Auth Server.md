---
tags:
  - developer/backend
---
# Auth Server

> Axum server on port 3001. Handles auth, sessions, and app distribution.

## Architecture

```
auth-server/
  src/
    lib.rs      - public module declarations
    main.rs     - Axum routes, handler functions (distro inlined here)
    auth.rs     - JWT creation/verification, argon2 password hashing
    db.rs       - SQLite queries (users, refresh_tokens, app_versions)
    models.rs   - Shared types (User, AuthResponse, TokenClaims, etc.)
  tests/
    auth_test.rs - Integration tests
```

## API Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/health` | Health check | No |
| POST | `/api/auth/register` | Register with email + password | No |
| POST | `/api/auth/login` | Login with email + password | No |
| POST | `/api/auth/refresh` | Exchange refresh token for new access token | No (has token) |
| POST | `/api/auth/verify` | Verify an access token is valid | No |
| POST | `/api/auth/google` | Exchange Google auth code for JWT | No (has code) |
| GET | `/api/distro/latest-version` | Get latest app version info | No |
| GET | `/api/distro/download/{version}` | Download app binary | No |

## Auth Flow

### Email/Password
```
POST /api/auth/register { email, password, name }
  → { access_token, refresh_token, user }

POST /api/auth/login { email, password }
  → { access_token, refresh_token, user }

POST /api/auth/refresh { refresh_token }
  → { access_token, refresh_token, user }
```

### Token Details
- **Access token**: JWT, 1 hour expiry
- **Refresh token**: Random 64-char hex, 30 day expiry, stored in DB
- **Password hashing**: Argon2id (memory-hard, OWASP recommended)

### Google OAuth (Pending)
- Desktop app uses localhost callback + PKCE flow
- `get_user_by_provider("google", provider_id)` links accounts
- Implementation: Sprint 2 continuation

## Database Tables

### users
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT | UUID v4 |
| email | TEXT | Unique |
| password_hash | TEXT | Nullable (NULL for OAuth users) |
| name | TEXT | Display name |
| avatar_url | TEXT | From OAuth provider |
| provider | TEXT | "email" or "google" or "apple" |
| provider_id | TEXT | Provider's user ID |
| created_at | TEXT | RFC 3339 |

### refresh_tokens
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT | UUID v4 |
| user_id | TEXT | FK to users |
| token | TEXT | Unique, 64-char hex |
| expires_at | TEXT | RFC 3339 |
| created_at | TEXT | RFC 3339 |

### app_versions (for distribution)
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER | Auto-increment |
| version | TEXT | SemVer |
| url | TEXT | Download URL |
| checksum | TEXT | SHA-256 |
| released_at | TEXT | RFC 3339 |

## Running

```bash
cargo run -p auth-server
# Listens on http://0.0.0.0:3001
```

## Tests

```bash
cargo test -p auth-server
# 6 integration tests: create user, duplicates, nonexistent, refresh flow, expired cleanup, provider lookup
```

## Google OAuth

### Flow
1. Frontend calls Tauri command `start_google_oauth`
2. Tauri starts TCP listener on `127.0.0.1:0`, opens browser to Google auth URL
3. Google redirects to `http://127.0.0.1:{port}/callback?code=...`
4. Tauri catches code, sends to backend
5. Backend exchanges code with Google, creates/finds user, returns JWT

### Env vars
| Variable | Required | Description |
|----------|----------|-------------|
| GOOGLE_CLIENT_ID | Yes | Google OAuth client ID |
| GOOGLE_CLIENT_SECRET | Yes | Google OAuth client secret |

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| axum | 0.7 | HTTP framework |
| tokio | 1.44 | Async runtime |
| jsonwebtoken | 9.3 | JWT creation/verification |
| argon2 | 0.5 | Password hashing |
| rusqlite | 0.31 | SQLite |
| reqwest | 0.12 | HTTP client (for OAuth token exchange) |
| tower-http | 0.5 | CORS middleware |

Related: [[02 - Architecture|Architecture]] | [[09 - Build & Deploy|Deploy]]
