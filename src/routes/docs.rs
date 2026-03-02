use axum::{
    extract::{ Query, State },
    http::StatusCode,
    response::{ Html, IntoResponse, Response },
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/docs", get(docs_page))
        .route("/docs/auth", get(docs_auth))
        .route("/docs/onboarding", get(docs_onboarding))
        .route("/docs/mood", get(docs_mood))
        .route("/docs/analytics", get(docs_analytics))
        .route("/docs/reports", get(docs_reports))
        .route("/docs/notes", get(docs_notes))
        .route("/docs/chats", get(docs_chats))
        .route("/docs/logs", get(docs_logs))
}

#[derive(Debug, Deserialize)]
struct DocsQuery {
    password: Option<String>,
}

/// Verify docs password and return error page if invalid.
fn check_password(state: &AppState, q: &DocsQuery) -> Option<Response> {
    match &q.password {
        Some(p) if p == &state.config.docs.password => None,
        _ =>
            Some(
                (
                    StatusCode::UNAUTHORIZED,
                    Html(
                        render_page(
                            "Access Denied",
                            r#"<div class="error-card">
                        <h2><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="rgb(248,81,73)" stroke-width="2" style="vertical-align:middle;margin-right:4px"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg> Password Required</h2>
                        <p>Append <code>?password=YOUR_DOCS_PASSWORD</code> to the URL.</p>
                    </div>"#
                        )
                    ),
                ).into_response()
            ),
    }
}

// ── Index ───────────────────────────────────────────────────

async fn docs_page(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg> BSDY API Documentation</h1>
        <p class="subtitle">Mental Companion &amp; Tracker Platform — v{version}</p>
        <p>Base URL: <code>{base}</code> &nbsp;|&nbsp; Mode: <code>{mode}</code></p>

        <div class="section-grid">
            <a class="card" href="/docs/auth?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg> Authentication</h3>
                <p>Google OAuth, JWT, email verification, profile</p>
            </a>
            <a class="card" href="/docs/onboarding?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M12 2a7 7 0 017 7c0 3.87-3.13 7-7 7s-7-3.13-7-7a7 7 0 017-7z"/><path d="M12 16v6"/><path d="M8 22h8"/></svg> Onboarding</h3>
                <p>Baseline mental health assessment</p>
            </a>
            <a class="card" href="/docs/mood?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg> Mood Tracker</h3>
                <p>Daily mood entries, history, today's mood</p>
            </a>
            <a class="card" href="/docs/analytics?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg> Analytics</h3>
                <p>AI-powered mood analytics summaries</p>
            </a>
            <a class="card" href="/docs/reports?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> Reports</h3>
                <p>Mental health report generation &amp; email delivery</p>
            </a>
            <a class="card" href="/docs/notes?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> Coping Toolkit</h3>
                <p>Notes CRUD with labels &amp; pinning</p>
            </a>
            <a class="card" href="/docs/chats?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg> Chat</h3>
                <p>Companion &amp; Agentic AI chatbot</p>
            </a>
            <a class="card" href="/docs/logs?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> Logs</h3>
                <p>Auth &amp; activity audit trail</p>
            </a>
        </div>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg> Authentication Flow</h2>
        <ol>
            <li><strong>GET</strong> <code>/api/auth/google/url</code> → redirect user to Google consent</li>
            <li>User consents → Google redirects with <code>code</code></li>
            <li><strong>POST</strong> <code>/api/auth/google/callback</code> with <code>{{"code":"..."}}</code> → returns JWT + user</li>
            <li>Use JWT as <code>Authorization: Bearer &lt;token&gt;</code> for all subsequent requests</li>
            <li>Verify email → complete onboarding → full access</li>
        </ol>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg> Access Levels</h2>
        <table>
            <tr><th>Extractor</th><th>Requires</th><th>Used By</th></tr>
            <tr><td><code>AuthUser</code></td><td>Valid JWT</td><td>GET /me, resend-verification, logs</td></tr>
            <tr><td><code>VerifiedUser</code></td><td>JWT + verified email</td><td>Onboarding, profile update</td></tr>
            <tr><td><code>FullUser</code></td><td>JWT + verified + onboarded</td><td>Mood, analytics, reports, notes, chats</td></tr>
        </table>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/></svg> Response Format</h2>
        <pre>{{"success": true, "data": {{ ... }} }}</pre>
        <pre>{{"success": false, "error": {{ "type": "BAD_REQUEST", "message": "..." }} }}</pre>
        "#,
        version = env!("CARGO_PKG_VERSION"),
        base = state.config.app.frontend_url,
        mode = state.config.app.mode,
        pw = pw
    );

    Html(render_page("BSDY API Docs", &body)).into_response()
}

// ── Auth Docs ───────────────────────────────────────────────

async fn docs_auth(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg> Authentication</h1>

        {endpoint_block}
        "#,
        pw = pw,
        endpoint_block = [
            endpoint(
                "GET",
                "/api/auth/google/url",
                "None",
                "Get the Google OAuth consent URL to redirect the user to.",
                r#"// No request body"#,
                r#"{{
  "success": true,
  "url": "https://accounts.google.com/o/oauth2/v2/auth?..."
}}"#
            ),
            endpoint(
                "POST",
                "/api/auth/google/callback",
                "None",
                "Exchange the Google auth code for a JWT token and user profile. Sends verification email for new users.",
                r#"{{ "code": "4/0AY0e-g7..." }}"#,
                r#"{{
  "success": true,
  "data": {{
    "token": "eyJhbGciOiJIUzI1NiJ9...",
    "user": {{
      "id": "uuid",
      "username": "johndoe",
      "name": "John Doe",
      "email": "john@gmail.com",
      "avatar_url": "https://...",
      "birth": null,
      "email_verified": false,
      "onboarding_completed": false,
      "created_at": "2026-01-15 10:30:00"
    }},
    "is_new_user": true
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/auth/verify-email?token=...",
                "None",
                "Verify user's email via the token sent in the verification email.",
                r#"// Query param: ?token=abc123..."#,
                r#"{{
  "success": true,
  "message": "Email verified successfully",
  "user": {{ ... }}
}}"#
            ),
            endpoint(
                "POST",
                "/api/auth/resend-verification",
                "AuthUser (JWT)",
                "Resend the verification email to the authenticated user.",
                r#"// No request body"#,
                r#"{{
  "success": true,
  "message": "Verification email sent"
}}"#
            ),
            endpoint(
                "GET",
                "/api/auth/me",
                "AuthUser (JWT)",
                "Get the current authenticated user's profile.",
                r#"// No request body"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "username": "johndoe",
    "name": "John Doe",
    "email": "john@gmail.com",
    "avatar_url": "https://...",
    "birth": "2000-05-15",
    "email_verified": true,
    "onboarding_completed": true,
    "created_at": "2026-01-15 10:30:00"
  }}
}}"#
            ),
            endpoint(
                "PUT",
                "/api/auth/me",
                "VerifiedUser (JWT + verified email)",
                "Update the current user's profile (name, birth date).",
                r#"{{
  "name": "John Updated",
  "birth": "2000-05-15"
}}"#,
                r#"{{
  "success": true,
  "data": {{ ... }}
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Auth — BSDY Docs", &body)).into_response()
}

// ── Onboarding Docs ─────────────────────────────────────────

async fn docs_onboarding(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M12 2a7 7 0 017 7c0 3.87-3.13 7-7 7s-7-3.13-7-7a7 7 0 017-7z"/><path d="M12 16v6"/><path d="M8 22h8"/></svg> Onboarding / Baseline Assessment</h1>
        <p>Users must complete a baseline mental health assessment after email verification.
        All sensitive data is encrypted at rest with per-user derived keys.</p>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/onboarding/baseline",
                "VerifiedUser",
                "Submit the initial baseline mental health assessment. Also sets the user's birth date and marks onboarding as complete.",
                r#"{{
  "birth": "2000-05-15",
  "family_background": "No significant history",
  "stress_level": "moderate",
  "anxiety_level": "low",
  "depression_level": "low",
  "sleep_quality": "moderate",
  "social_support": "strong",
  "coping_style": "problem_focused",
  "personality_traits": "[\"empathetic\",\"introverted\"]",
  "mental_health_history": "No prior diagnoses",
  "current_medications": null,
  "therapy_status": "none",
  "additional_notes": null
}}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "user_id": "uuid",
    "risk_level": "low",
    "stress_level": "moderate",
    "anxiety_level": "low",
    ...all fields decrypted...
    "created_at": "2026-01-15 10:30:00"
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/onboarding/baseline",
                "VerifiedUser",
                "Get the user's current baseline assessment (decrypted).",
                "// No request body",
                r#"{{ "success": true, "data": {{ ...baseline... }} }}"#
            ),
            endpoint(
                "PUT",
                "/api/onboarding/baseline",
                "VerifiedUser",
                "Update specific fields of the baseline. Only provided fields are changed.",
                r#"{{
  "stress_level": "high",
  "therapy_status": "active"
}}"#,
                r#"{{ "success": true, "data": {{ ...updated baseline... }} }}"#
            ),
        ].join("\n")
    );

    Html(render_page("Onboarding — BSDY Docs", &body)).into_response()
}

// ── Mood Docs ───────────────────────────────────────────────

async fn docs_mood(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg> Mood Tracker</h1>
        <p>One entry per day. If you POST on the same day, it updates the existing entry (upsert).</p>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/mood",
                "FullUser",
                "Create or update today's mood entry.",
                r#"{{
  "mood_score": 7,
  "energy_level": 6,
  "anxiety_level": 3,
  "stress_level": 4,
  "sleep_hours": 7.5,
  "sleep_quality": 7,
  "appetite": "normal",
  "social_interaction": true,
  "exercise_done": false,
  "notes": "Felt productive today",
  "triggers": "[\"work deadline\"]",
  "activities": "[\"reading\",\"walking\"]"
}}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "entry_date": "2026-03-02",
    "mood_score": 7,
    "energy_level": 6,
    ... all fields ...
    "created_at": "2026-03-02 08:30:00"
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/mood?from=2026-02-01&to=2026-02-28&limit=30",
                "FullUser",
                "Get mood entries for a date range. Defaults to last 30 days, max 90.",
                "// Query params: from, to (YYYY-MM-DD), limit",
                r#"{{
  "success": true,
  "data": [ {{ ...entry... }}, ... ],
  "count": 28
}}"#
            ),
            endpoint(
                "GET",
                "/api/mood/today",
                "FullUser",
                "Check if today's mood has been logged.",
                "// No request body",
                r#"{{
  "success": true,
  "data": {{ ...entry or null... }},
  "logged_today": true
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Mood — BSDY Docs", &body)).into_response()
}

// ── Analytics Docs ──────────────────────────────────────────

async fn docs_analytics(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg> Analytics</h1>
        <p>AI-powered analysis of mood data using Gemini. Generates insights, recommendations, trend analysis.</p>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/analytics/generate",
                "FullUser",
                "Generate a new AI analytics summary for the specified period.",
                r#"{{ "period_type": "weekly" }}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "period_type": "weekly",
    "period_start": "2026-02-23",
    "period_end": "2026-03-02",
    "summary": "Your mood has been...",
    "insights": "Key patterns include...",
    "recommendations": "Consider trying...",
    "overall_mood_trend": "improving",
    "avg_mood_score": 6.5,
    "risk_level": "low",
    "generated_by": "manual",
    "created_at": "2026-03-02 10:00:00"
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/analytics?limit=10",
                "FullUser",
                "Get previously generated analytics summaries.",
                "// Query param: limit (default 10)",
                r#"{{ "success": true, "data": [ ... ], "count": 5 }}"#
            ),
        ].join("\n")
    );

    Html(render_page("Analytics — BSDY Docs", &body)).into_response()
}

// ── Reports Docs ────────────────────────────────────────────

async fn docs_reports(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> Mental Health Reports</h1>
        <p>Comprehensive AI-generated reports. Can be emailed automatically (weekly) or generated on demand.</p>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/reports/generate",
                "FullUser",
                "Generate a new mental health report. Optionally send via email.",
                r#"{{
  "report_type": "weekly",
  "period_start": null,
  "period_end": null,
  "send_email": true
}}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "report_type": "weekly",
    "period_start": "2026-02-23",
    "period_end": "2026-03-02",
    "title": "Weekly Mental Health Report",
    "content": "...",
    "ai_analysis": "...",
    "recommendations": "...",
    "status": "completed",
    "sent_via_email": true,
    "trigger_type": "manual",
    "created_at": "2026-03-02 10:00:00"
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/reports?limit=10",
                "FullUser",
                "List generated reports.",
                "// Query param: limit (default 10)",
                r#"{{ "success": true, "data": [ ... ], "count": 3 }}"#
            ),
            endpoint(
                "GET",
                "/api/reports/:report_id",
                "FullUser",
                "Get a specific report by ID.",
                "// Path param: report_id",
                r#"{{ "success": true, "data": {{ ...report... }} }}"#
            ),
        ].join("\n")
    );

    Html(render_page("Reports — BSDY Docs", &body)).into_response()
}

// ── Notes Docs ──────────────────────────────────────────────

async fn docs_notes(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> Coping Toolkit (Notes)</h1>
        <p>Encrypted personal notes with labels and pinning. All content is E2E encrypted at rest.</p>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/notes",
                "FullUser",
                "Create a new note.",
                r#"{{
  "title": "Breathing exercise",
  "content": "4-7-8 technique: breathe in 4s, hold 7s, out 8s",
  "label": "coping",
  "is_pinned": true
}}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "title": "Breathing exercise",
    "content": "4-7-8 technique...",
    "label": "coping",
    "is_pinned": true,
    "created_at": "...",
    "updated_at": "..."
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/notes?label=coping&limit=50",
                "FullUser",
                "List notes. Optionally filter by label.",
                "// Query params: label (optional), limit (default 50)",
                r#"{{ "success": true, "data": [ ... ], "count": 12 }}"#
            ),
            endpoint(
                "GET",
                "/api/notes/labels",
                "FullUser",
                "Get all distinct labels used by the user.",
                "// No request body",
                r#"{{ "success": true, "data": ["coping", "journal", "gratitude"] }}"#
            ),
            endpoint(
                "GET",
                "/api/notes/:note_id",
                "FullUser",
                "Get a specific note.",
                "// Path param: note_id",
                r#"{{ "success": true, "data": {{ ...note... }} }}"#
            ),
            endpoint(
                "PUT",
                "/api/notes/:note_id",
                "FullUser",
                "Update a note. Only provided fields are changed.",
                r#"{{ "title": "Updated title", "is_pinned": false }}"#,
                r#"{{ "success": true, "data": {{ ...updated note... }} }}"#
            ),
            endpoint(
                "DELETE",
                "/api/notes/:note_id",
                "FullUser",
                "Delete a note.",
                "// Path param: note_id",
                r#"{{ "success": true, "message": "Note deleted" }}"#
            ),
        ].join("\n")
    );

    Html(render_page("Notes — BSDY Docs", &body)).into_response()
}

// ── Chats Docs ──────────────────────────────────────────────

async fn docs_chats(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg> Chat</h1>
        <p>Two chat modes:</p>
        <ul>
            <li><strong>companion</strong> — Empathetic AI companion (default). Crisis-aware.</li>
            <li><strong>agentic</strong> — AI with tool access to mood data, analytics, notes, reports.</li>
        </ul>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "POST",
                "/api/chats",
                "FullUser",
                "Create a new chat session.",
                r#"{{ "chat_type": "companion" }}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "title": "New Chat",
    "chat_type": "companion",
    "is_active": true,
    "message_count": 0,
    "created_at": "...",
    "updated_at": "..."
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/chats?limit=20",
                "FullUser",
                "List chat sessions.",
                "// Query param: limit (default 20)",
                r#"{{ "success": true, "data": [ ... ], "count": 5 }}"#
            ),
            endpoint(
                "GET",
                "/api/chats/:chat_id",
                "FullUser",
                "Get a specific chat.",
                "// Path param: chat_id",
                r#"{{ "success": true, "data": {{ ...chat... }} }}"#
            ),
            endpoint(
                "PUT",
                "/api/chats/:chat_id",
                "FullUser",
                "Update chat title or active status.",
                r#"{{ "title": "My Session", "is_active": false }}"#,
                r#"{{ "success": true, "data": {{ ...updated chat... }} }}"#
            ),
            endpoint(
                "DELETE",
                "/api/chats/:chat_id",
                "FullUser",
                "Delete a chat and all its messages.",
                "// Path param: chat_id",
                r#"{{ "success": true, "message": "Chat deleted" }}"#
            ),
            endpoint(
                "GET",
                "/api/chats/:chat_id/messages?limit=50",
                "FullUser",
                "Get decrypted message history for a chat.",
                "// Query param: limit (default 50)",
                r#"{{ "success": true, "data": [ ... ], "count": 24 }}"#
            ),
            endpoint(
                "POST",
                "/api/chats/:chat_id/messages",
                "FullUser",
                "Send a message. Routes to companion or agentic based on chat type. Returns both user and assistant messages.",
                r#"{{ "message": "How am I doing this week?" }}"#,
                r#"{{
  "success": true,
  "data": {{
    "user_message": {{
      "id": "uuid",
      "chat_id": "uuid",
      "role": "user",
      "content": "How am I doing this week?",
      "tool_calls": null,
      "tool_results": null,
      "has_tool_calls": false,
      "severity_flag": "none",
      "created_at": "..."
    }},
    "assistant_message": {{
      "id": "uuid",
      "chat_id": "uuid",
      "role": "assistant",
      "content": "Based on your mood entries...",
      "tool_calls": [...],
      "tool_results": [...],
      "has_tool_calls": true,
      "severity_flag": "none",
      "created_at": "..."
    }}
  }}
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Chat — BSDY Docs", &body)).into_response()
}

// ── Logs Docs ───────────────────────────────────────────────

async fn docs_logs(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">← Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> Audit Logs</h1>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "GET",
                "/api/logs/auth?page=1&per_page=20",
                "AuthUser",
                "Paginated authentication event log (login, email_verify, verification_sent).",
                "// Query params: page (default 1), per_page (default 20, max 100)",
                r#"{{
  "success": true,
  "data": {{
    "data": [
      {{
        "id": "uuid",
        "action": "login",
        "ip_address": null,
        "success": true,
        "failure_reason": null,
        "created_at": "..."
      }}
    ],
    "total": 42,
    "page": 1,
    "per_page": 20
  }}
}}"#
            ),
            endpoint(
                "GET",
                "/api/logs/activity?page=1&per_page=20&feature=mood_tracker",
                "AuthUser",
                "Paginated activity log. Optionally filter by feature.",
                "// Query params: page, per_page, feature (optional)",
                r#"{{
  "success": true,
  "data": {{
    "data": [
      {{
        "id": "uuid",
        "action": "create",
        "feature": "mood_tracker",
        "entity_type": "mood_entry",
        "entity_id": "uuid",
        "details": null,
        "created_at": "..."
      }}
    ],
    "total": 150,
    "page": 1,
    "per_page": 20
  }}
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Logs — BSDY Docs", &body)).into_response()
}

// ── Helpers ─────────────────────────────────────────────────

fn endpoint(
    method: &str,
    path: &str,
    auth: &str,
    description: &str,
    request_example: &str,
    response_example: &str
) -> String {
    let method_class = method.to_lowercase();
    format!(
        r#"
        <div class="endpoint">
            <div class="endpoint-header">
                <span class="method {method_class}">{method}</span>
                <code class="path">{path}</code>
                <span class="auth-badge">{auth}</span>
            </div>
            <p>{description}</p>
            <div class="code-blocks">
                <div>
                    <h4>Request</h4>
                    <pre>{request_example}</pre>
                </div>
                <div>
                    <h4>Response</h4>
                    <pre>{response_example}</pre>
                </div>
            </div>
        </div>
        "#
    )
}

fn render_page(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        :root {{
            --bg: #0d1117;
            --surface: #161b22;
            --border: #30363d;
            --text: #e6edf3;
            --text-muted: #8b949e;
            --accent: #58a6ff;
            --green: #3fb950;
            --yellow: #d29922;
            --red: #f85149;
            --purple: #bc8cff;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding: 2rem;
            max-width: 960px;
            margin: 0 auto;
        }}
        h1 {{ margin-bottom: 0.5rem; }}
        h2 {{ margin-top: 2rem; margin-bottom: 0.75rem; color: var(--accent); border-bottom: 1px solid var(--border); padding-bottom: 0.3rem; }}
        .subtitle {{ color: var(--text-muted); margin-bottom: 1rem; }}
        a {{ color: var(--accent); text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        code {{ background: var(--surface); padding: 0.15rem 0.4rem; border-radius: 4px; font-size: 0.9em; }}
        pre {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 1rem;
            overflow-x: auto;
            font-size: 0.85em;
            line-height: 1.5;
            white-space: pre-wrap;
        }}
        table {{ width: 100%; border-collapse: collapse; margin: 1rem 0; }}
        th, td {{ text-align: left; padding: 0.5rem 0.75rem; border: 1px solid var(--border); }}
        th {{ background: var(--surface); }}
        ol, ul {{ padding-left: 1.5rem; margin: 0.5rem 0; }}
        li {{ margin: 0.3rem 0; }}

        .section-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
            gap: 1rem;
            margin: 1.5rem 0;
        }}
        .card {{
            display: block;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 1.2rem;
            transition: border-color 0.2s;
        }}
        .card:hover {{ border-color: var(--accent); text-decoration: none; }}
        .card h3 {{ margin-bottom: 0.3rem; }}
        .card p {{ color: var(--text-muted); font-size: 0.9em; }}

        .endpoint {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 1.25rem;
            margin: 1rem 0;
        }}
        .endpoint-header {{ display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; margin-bottom: 0.5rem; }}
        .method {{
            font-weight: 700;
            font-size: 0.8em;
            padding: 0.2rem 0.6rem;
            border-radius: 4px;
            color: #fff;
        }}
        .method.get {{ background: var(--green); }}
        .method.post {{ background: var(--accent); }}
        .method.put {{ background: var(--yellow); color: #000; }}
        .method.delete {{ background: var(--red); }}
        .path {{ font-size: 0.95em; background: transparent; padding: 0; }}
        .auth-badge {{
            font-size: 0.75em;
            padding: 0.15rem 0.5rem;
            border-radius: 10px;
            border: 1px solid var(--purple);
            color: var(--purple);
            margin-left: auto;
        }}
        .code-blocks {{ display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; margin-top: 0.75rem; }}
        .code-blocks h4 {{ font-size: 0.8em; color: var(--text-muted); margin-bottom: 0.25rem; }}
        @media (max-width: 700px) {{ .code-blocks {{ grid-template-columns: 1fr; }} }}

        .error-card {{
            background: var(--surface);
            border: 1px solid var(--red);
            border-radius: 8px;
            padding: 2rem;
            text-align: center;
            margin-top: 4rem;
        }}
        .error-card h2 {{ color: var(--red); border: none; }}
    </style>
</head>
<body>
    {body}
</body>
</html>"#
    )
}
