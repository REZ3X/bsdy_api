# BSDY API

Mental Companion & Tracker Platform — backend API built with **Rust / Axum**.

---

## Table of Contents

- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
  - [1. Clone the Repository](#1-clone-the-repository)
  - [2. Configure Environment](#2-configure-environment)
  - [3. Set Up the Database](#3-set-up-the-database)
  - [4. Run the Server](#4-run-the-server)
- [API Endpoints](#api-endpoints)
  - [Health](#health)
  - [Authentication](#authentication)
  - [Onboarding](#onboarding)
  - [Mood Tracker](#mood-tracker)
  - [Analytics](#analytics)
  - [Reports](#reports)
  - [Notes](#notes)
  - [Chats](#chats)
  - [Logs](#logs)
  - [Documentation Pages](#documentation-pages)
- [Authentication Flow](#authentication-flow)
- [Encryption](#encryption)
- [App Modes](#app-modes)
- [Background Scheduler](#background-scheduler)
- [Testing](#testing)
  - [Running Tests](#running-tests)
  - [Test Categories](#test-categories)
- [Environment Variables Reference](#environment-variables-reference)
- [Deployment](#deployment)
- [License](#license)

---

## Features

- **Google OAuth 2.0** authentication with JWT session tokens
- **Email verification** via Brevo SMTP
- **Gemini AI** integration for mental-health chat companion and agentic actions
- **End-to-end at-rest encryption** (AES-256-GCM + HKDF per-user key derivation)
- **Mood tracking** with daily entries and trend analytics
- **AI-generated analytics & reports** (weekly / monthly / yearly, scheduled or on-demand)
- **Encrypted notes** (coping toolkit)
- **Chat system** with AI companion and agentic tool-calling capabilities
- **Background scheduler** for automated report generation
- **Password-protected API docs** served as HTML pages
- **Dual mode**: `internal` (relaxed) and `external` (API-key gated)
- **Activity & auth logging** for auditing
- **Graceful shutdown** with Ctrl+C / SIGTERM handling

---

## Tech Stack

| Layer         | Technology                                   |
| ------------- | -------------------------------------------- |
| Language      | Rust 2021 edition                            |
| Web framework | Axum 0.7                                     |
| Async runtime | Tokio                                        |
| Database      | MariaDB / MySQL via SQLx 0.8                 |
| Auth          | Google OAuth 2.0, jsonwebtoken 9             |
| Email         | Brevo SMTP via lettre 0.11                   |
| AI            | Google Gemini API via reqwest 0.12           |
| Encryption    | AES-256-GCM (aes-gcm 0.10), HKDF (hkdf 0.12) |
| Scheduling    | tokio-cron-scheduler 0.11                    |
| Logging       | tracing + tracing-subscriber                 |

---

## Project Structure

```
bsdy_api/
├── Cargo.toml                # Dependencies & metadata
├── .env.example              # Environment template (copy to .env)
├── migrations/
│   └── 001_initial_schema.sql
├── src/
│   ├── main.rs               # Entry point — server startup
│   ├── lib.rs                # Public module re-exports
│   ├── config.rs             # Configuration structs + env loading
│   ├── crypto.rs             # AES-256-GCM encryption service
│   ├── db.rs                 # Database pool creation + migration runner
│   ├── error.rs              # Unified AppError type
│   ├── state.rs              # Shared AppState (pool, config, services)
│   ├── middleware/
│   │   ├── api_key.rs        # API-key gate for external mode
│   │   ├── auth.rs           # JWT AuthUser / VerifiedUser extractors
│   │   └── activity_log.rs   # Request logging helpers
│   ├── models/
│   │   ├── user.rs           # UserRow, Claims, AuthResponse, etc.
│   │   ├── mental.rs         # Baseline assessment models
│   │   ├── chat.rs           # Chat & message models
│   │   ├── note.rs           # Note models
│   │   └── log.rs            # Auth & activity log models
│   ├── routes/
│   │   ├── mod.rs            # build_router() — assembles all routes
│   │   ├── auth.rs           # /api/auth/*
│   │   ├── onboarding.rs     # /api/onboarding/*
│   │   ├── mood.rs           # /api/mood/*
│   │   ├── analytics.rs      # /api/analytics/*
│   │   ├── report.rs         # /api/reports/*
│   │   ├── note.rs           # /api/notes/*
│   │   ├── chat.rs           # /api/chats/*
│   │   ├── log.rs            # /api/logs/*
│   │   ├── health.rs         # /health
│   │   └── docs.rs           # /docs/*
│   └── services/
│       ├── auth_service.rs
│       ├── onboarding_service.rs
│       ├── mood_service.rs
│       ├── analytics_service.rs
│       ├── report_service.rs
│       ├── note_service.rs
│       ├── chat_service.rs
│       ├── agent_service.rs
│       ├── gemini_service.rs
│       ├── email_service.rs
│       └── scheduler_service.rs
└── tests/
    ├── common/mod.rs          # Shared test helpers
    ├── test_config_crypto.rs  # Config parsing & encryption tests
    ├── test_errors_models.rs  # Error types & model tests
    ├── test_auth_chat.rs      # JWT, auth service, chat model tests
    ├── test_database.rs       # DB pool, migrations, CRUD queries
    ├── test_gemini.rs         # Gemini service unit & live tests
    ├── test_email.rs          # Email service unit & live tests
    ├── test_services.rs       # Service-layer integration tests
    └── test_routes.rs         # HTTP route & scheduler tests
```

---

## Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **MariaDB** 10.6+ or **MySQL** 8.0+
- **Google Cloud** project with OAuth 2.0 credentials
- **Brevo** account for SMTP email delivery
- **Google Gemini** API key

---

## Getting Started

### 1. Clone the Repository

```bash
git clone <repo-url>
cd bsdy_api
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` and fill in your real credentials. See [Environment Variables Reference](#environment-variables-reference) for details.

> **Important:** Cron values that contain `*` must be wrapped in double quotes in the `.env` file, e.g. `WEEKLY_REPORT_CRON="0 0 9 * * Mon"`.

### 3. Set Up the Database

Create the database and user in MariaDB:

```sql
CREATE DATABASE bsdy_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'bsdy_user'@'%' IDENTIFIED BY 'your_password';
GRANT ALL PRIVILEGES ON bsdy_db.* TO 'bsdy_user'@'%';
FLUSH PRIVILEGES;
```

Migrations run **automatically** on server start. The migration runner in `src/db.rs` reads every `.sql` file under `migrations/` and executes them in order. You can also apply them manually:

```bash
mysql -u bsdy_user -p bsdy_db < migrations/001_initial_schema.sql
```

### 4. Run the Server

```bash
# Development (with hot-reload logging)
cargo run

# Release build
cargo build --release
./target/release/bsdy_api
```

The server listens on `0.0.0.0:{APP_PORT}` (default `8000`).

Verify it's running:

```bash
curl http://localhost:8000/health
```

Expected response:

```json
{
  "success": true,
  "status": "healthy",
  "service": "bsdy-api",
  "version": "0.2.0",
  "database": "connected"
}
```

---

## API Endpoints

All protected routes require a `Authorization: Bearer <JWT>` header. Routes under `/api/auth/me`, `/api/onboarding/*`, `/api/mood/*`, `/api/analytics/*`, `/api/reports/*`, `/api/notes/*`, `/api/chats/*`, and `/api/logs/*` also require a **verified email**.

### Health

| Method | Path      | Auth | Description                    |
| ------ | --------- | ---- | ------------------------------ |
| GET    | `/health` | No   | Server & database health check |

### Authentication

| Method | Path                            | Auth           | Description                  |
| ------ | ------------------------------- | -------------- | ---------------------------- |
| GET    | `/api/auth/google/url`          | No             | Get Google OAuth consent URL |
| POST   | `/api/auth/google/callback`     | No             | Exchange auth code for JWT   |
| GET    | `/api/auth/verify-email`        | No             | Verify email via token link  |
| POST   | `/api/auth/resend-verification` | JWT            | Resend verification email    |
| GET    | `/api/auth/me`                  | JWT + Verified | Get current user profile     |
| PUT    | `/api/auth/me`                  | JWT + Verified | Update user profile          |

### Onboarding

| Method | Path                       | Auth           | Description                            |
| ------ | -------------------------- | -------------- | -------------------------------------- |
| POST   | `/api/onboarding/baseline` | JWT + Verified | Save baseline mental-health assessment |
| GET    | `/api/onboarding/baseline` | JWT + Verified | Get current baseline                   |
| PUT    | `/api/onboarding/baseline` | JWT + Verified | Update baseline assessment             |

### Mood Tracker

| Method | Path              | Auth           | Description                                   |
| ------ | ----------------- | -------------- | --------------------------------------------- |
| POST   | `/api/mood`       | JWT + Verified | Create or update today's mood entry           |
| GET    | `/api/mood`       | JWT + Verified | List mood entries (supports date range query) |
| GET    | `/api/mood/today` | JWT + Verified | Get today's mood entry                        |

### Analytics

| Method | Path                      | Auth           | Description                   |
| ------ | ------------------------- | -------------- | ----------------------------- |
| POST   | `/api/analytics/generate` | JWT + Verified | Generate AI analytics summary |
| GET    | `/api/analytics`          | JWT + Verified | List analytics summaries      |

### Reports

| Method | Path                       | Auth           | Description                         |
| ------ | -------------------------- | -------------- | ----------------------------------- |
| POST   | `/api/reports/generate`    | JWT + Verified | Generate an AI mental-health report |
| GET    | `/api/reports`             | JWT + Verified | List all reports                    |
| GET    | `/api/reports/{report_id}` | JWT + Verified | Get a specific report               |

### Notes

| Method | Path                   | Auth           | Description               |
| ------ | ---------------------- | -------------- | ------------------------- |
| POST   | `/api/notes`           | JWT + Verified | Create a new note         |
| GET    | `/api/notes`           | JWT + Verified | List all notes            |
| GET    | `/api/notes/labels`    | JWT + Verified | List distinct note labels |
| GET    | `/api/notes/{note_id}` | JWT + Verified | Get a specific note       |
| PUT    | `/api/notes/{note_id}` | JWT + Verified | Update a note             |
| DELETE | `/api/notes/{note_id}` | JWT + Verified | Delete a note             |

### Chats

| Method | Path                            | Auth           | Description                        |
| ------ | ------------------------------- | -------------- | ---------------------------------- |
| POST   | `/api/chats`                    | JWT + Verified | Create a new chat session          |
| GET    | `/api/chats`                    | JWT + Verified | List chat sessions                 |
| GET    | `/api/chats/{chat_id}`          | JWT + Verified | Get chat details                   |
| PUT    | `/api/chats/{chat_id}`          | JWT + Verified | Update chat (title, active status) |
| DELETE | `/api/chats/{chat_id}`          | JWT + Verified | Delete a chat                      |
| GET    | `/api/chats/{chat_id}/messages` | JWT + Verified | Get chat messages                  |
| POST   | `/api/chats/{chat_id}/messages` | JWT + Verified | Send a message (AI responds)       |

### Logs

| Method | Path                 | Auth           | Description             |
| ------ | -------------------- | -------------- | ----------------------- |
| GET    | `/api/logs/auth`     | JWT + Verified | Get authentication logs |
| GET    | `/api/logs/activity` | JWT + Verified | Get activity logs       |

### Documentation Pages

| Method | Path                                        | Auth           | Description         |
| ------ | ------------------------------------------- | -------------- | ------------------- |
| GET    | `/docs?password=<DOCS_PASSWORD>`            | Password query | API docs index      |
| GET    | `/docs/auth?password=<DOCS_PASSWORD>`       | Password query | Auth endpoints docs |
| GET    | `/docs/onboarding?password=<DOCS_PASSWORD>` | Password query | Onboarding docs     |
| GET    | `/docs/mood?password=<DOCS_PASSWORD>`       | Password query | Mood tracker docs   |
| GET    | `/docs/analytics?password=<DOCS_PASSWORD>`  | Password query | Analytics docs      |
| GET    | `/docs/reports?password=<DOCS_PASSWORD>`    | Password query | Reports docs        |
| GET    | `/docs/notes?password=<DOCS_PASSWORD>`      | Password query | Notes docs          |
| GET    | `/docs/chats?password=<DOCS_PASSWORD>`      | Password query | Chats docs          |
| GET    | `/docs/logs?password=<DOCS_PASSWORD>`       | Password query | Logs docs           |

---

## Authentication Flow

```
1. Client → GET /api/auth/google/url
   ← { "auth_url": "https://accounts.google.com/o/oauth2/v2/auth?..." }

2. User completes Google consent in browser

3. Client → POST /api/auth/google/callback  { "code": "<auth_code>" }
   ← { "token": "<JWT>", "user": { ... }, "is_new_user": true/false }

4. New users receive a verification email automatically

5. Client → GET /api/auth/verify-email?token=<token>
   ← Email verified, user can now access protected routes

6. All subsequent requests include:
   Authorization: Bearer <JWT>
```

---

## Encryption

All sensitive user data is encrypted **at rest** using AES-256-GCM:

- A global **master key** (`ENCRYPTION_MASTER_KEY`, 256-bit hex) is set in the environment
- Each user gets a unique random **salt** stored in `users.encryption_salt`
- A per-user encryption key is derived via **HKDF-SHA256**: `HKDF(master_key, user_salt)`
- Fields ending in `_enc` in the database are AES-256-GCM ciphertexts, base64-encoded
- Encrypted fields: baseline assessment, mood notes/triggers/activities, analytics summaries, report content, chat messages, note titles/content

---

## App Modes

| Mode         | `APP_MODE` | Behavior                                                                                                |
| ------------ | ---------- | ------------------------------------------------------------------------------------------------------- |
| **Internal** | `internal` | No API key required, relaxed CORS (for local dev / trusted network)                                     |
| **External** | `external` | Requires `X-API-Key` header on all requests (except `/api/auth/*`, `/docs/*`, `/health`), stricter CORS |

Set `APP_MODE=external` and `API_KEY=<your-key>` in `.env` for production.

---

## Background Scheduler

The server starts a background scheduler on boot that runs automated tasks:

| Task           | Default Schedule | Description                 |
| -------------- | ---------------- | --------------------------- |
| Weekly report  | `0 0 9 * * Mon`  | Every Monday at 09:00       |
| Monthly report | `0 0 9 1 * *`    | 1st of every month at 09:00 |
| Yearly report  | `0 0 9 1 1 *`    | January 1st at 09:00        |

Cron expressions use 6-field format: `sec min hour day-of-month month day-of-week`.

Configure via `WEEKLY_REPORT_CRON`, `MONTHLY_REPORT_CRON`, `YEARLY_REPORT_CRON` in `.env`.

---

## Testing

### Running Tests

```bash
# Run all unit tests (no external services needed)
cargo test

# Run all tests including integration tests that require MariaDB + live services
cargo test -- --include-ignored

# Run a specific test file
cargo test --test test_routes

# Run a specific test by name
cargo test test_health_endpoint -- --ignored

# Run with output printed (useful for debugging)
cargo test -- --nocapture

# Run with output for a specific ignored test
cargo test test_email_live_send_verification -- --ignored --nocapture
```

### Test Categories

| File                    | Tests                                                                | Requires                                                   |
| ----------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------- |
| `test_config_crypto.rs` | Config parsing, env defaults, crypto encrypt/decrypt, key derivation | Nothing                                                    |
| `test_errors_models.rs` | Error types, model serialization, request validation                 | Nothing                                                    |
| `test_auth_chat.rs`     | JWT encode/decode, auth service logic, chat models                   | Nothing                                                    |
| `test_database.rs`      | DB pool, migrations, CRUD on all tables                              | MariaDB                                                    |
| `test_gemini.rs`        | Request/response parsing (unit), live AI calls (ignored)             | Gemini API key (ignored tests)                             |
| `test_email.rs`         | Template rendering (unit), live SMTP send (ignored)                  | Brevo credentials + `TEST_EMAIL_RECIPIENT` (ignored tests) |
| `test_services.rs`      | Service-layer integration (mood, analytics, reports)                 | MariaDB                                                    |
| `test_routes.rs`        | HTTP endpoints, auth guards, scheduler cron validation               | MariaDB (ignored), nothing (cron tests)                    |

Tests marked `#[ignore]` require live external services (database, SMTP, Gemini API). Run them with `--include-ignored` or `--ignored`.

> **Note:** Tests load `.env` automatically using `CARGO_MANIFEST_DIR` so they work regardless of your working directory.

---

## Environment Variables Reference

| Variable                   | Required | Default                                          | Description                                                                   |
| -------------------------- | -------- | ------------------------------------------------ | ----------------------------------------------------------------------------- |
| `APP_NAME`                 | No       | `BSDY`                                           | Application name (used in emails)                                             |
| `APP_ENV`                  | No       | `development`                                    | Environment: `development` / `production`                                     |
| `APP_PORT`                 | No       | `8000`                                           | Server listen port                                                            |
| `APP_MODE`                 | No       | `internal`                                       | `internal` or `external` (API key gating)                                     |
| `FRONTEND_URL`             | No       | `http://localhost:3000`                          | Frontend URL for CORS & email links                                           |
| `DATABASE_URL`             | **Yes**  | —                                                | MariaDB connection string: `mariadb://user:pass@host:port/db`                 |
| `DATABASE_MAX_CONNECTIONS` | No       | `10`                                             | Connection pool size                                                          |
| `JWT_SECRET`               | **Yes**  | —                                                | Secret for signing JWTs                                                       |
| `JWT_EXPIRATION_HOURS`     | No       | `72`                                             | JWT token lifetime in hours                                                   |
| `GOOGLE_CLIENT_ID`         | **Yes**  | —                                                | Google OAuth client ID                                                        |
| `GOOGLE_CLIENT_SECRET`     | **Yes**  | —                                                | Google OAuth client secret                                                    |
| `GOOGLE_REDIRECT_URI`      | No       | `http://localhost:8000/api/auth/google/callback` | OAuth callback URL                                                            |
| `BREVO_SMTP_HOST`          | No       | `smtp-relay.brevo.com`                           | SMTP server hostname                                                          |
| `BREVO_SMTP_PORT`          | No       | `587`                                            | SMTP server port                                                              |
| `BREVO_SMTP_USER`          | **Yes**  | —                                                | SMTP username                                                                 |
| `BREVO_SMTP_PASS`          | **Yes**  | —                                                | SMTP password                                                                 |
| `BREVO_FROM_EMAIL`         | No       | `noreply@bsdy.app`                               | Sender email address                                                          |
| `BREVO_FROM_NAME`          | No       | `BSDY Mental Companion`                          | Sender display name                                                           |
| `GEMINI_API_KEY`           | **Yes**  | —                                                | Google Gemini API key                                                         |
| `GEMINI_MODEL`             | No       | `gemini-3-flash-preview`                         | Gemini model identifier                                                       |
| `ENCRYPTION_MASTER_KEY`    | **Yes**  | —                                                | 64-char hex (256-bit) master encryption key. Generate: `openssl rand -hex 32` |
| `API_KEY`                  | No       | `""`                                             | API key for external mode (`X-API-Key` header)                                |
| `DOCS_PASSWORD`            | No       | `bsdy-docs-2026`                                 | Password for `/docs` pages                                                    |
| `WEEKLY_REPORT_CRON`       | No       | `0 0 9 * * Mon`                                  | Weekly report schedule (6-field cron)                                         |
| `MONTHLY_REPORT_CRON`      | No       | `0 0 9 1 * *`                                    | Monthly report schedule                                                       |
| `YEARLY_REPORT_CRON`       | No       | `0 0 9 1 1 *`                                    | Yearly report schedule                                                        |
| `TEST_EMAIL_RECIPIENT`     | No       | —                                                | Real email for live email tests                                               |

---

## Deployment

### Build for production

```bash
cargo build --release
```

The binary is at `target/release/bsdy_api` (Linux/macOS) or `target\release\bsdy_api.exe` (Windows).

### Run with environment

```bash
APP_ENV=production APP_MODE=external ./target/release/bsdy_api
```

Or use the `.env` file (it is loaded automatically).

### Docker (example)

```dockerfile
FROM rust:1.82 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bsdy_api /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations
WORKDIR /app
EXPOSE 8000
CMD ["bsdy_api"]
```

### Checklist

- [ ] Set `APP_ENV=production` and `APP_MODE=external`
- [ ] Use a strong, unique `JWT_SECRET`
- [ ] Generate a real 256-bit `ENCRYPTION_MASTER_KEY` (`openssl rand -hex 32`)
- [ ] Set a strong `API_KEY` for external mode
- [ ] Configure `FRONTEND_URL` to your actual frontend domain
- [ ] Set `GOOGLE_REDIRECT_URI` to your production callback URL
- [ ] Use real Brevo SMTP credentials
- [ ] Secure your MariaDB instance (not publicly accessible)
- [ ] Run behind a reverse proxy (nginx / Caddy) with TLS

---

## License

This project is part of a competition entry for TECHSOFT 2026.
