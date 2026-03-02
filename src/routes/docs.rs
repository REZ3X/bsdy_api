use axum::{
    extract::{ Query, State },
    http::StatusCode,
    response::{ Html, IntoResponse, Response },
    routing::{ get, post },
    Json,
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
        .route("/docs/content", get(docs_content))
        .route("/docs/logs", get(docs_logs))
        .route("/docs/playground", get(docs_playground))
        .route("/docs/tests", get(docs_tests))
        .route("/docs/run-tests", post(run_tests_handler))
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

// ── Index Page ──────────────────────────────────────────────

async fn docs_page(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <h1><svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> BSDY API Documentation</h1>
        <p class="subtitle">v{version} &middot; Base: <code>{base}</code> &middot; Mode: <code>{mode}</code></p>

        <h2>API Sections</h2>
        <div class="section-grid">
            <a class="card" href="/docs/auth?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg> Authentication</h3>
                <p>Google OAuth, JWT tokens, email verification</p>
            </a>
            <a class="card" href="/docs/onboarding?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M12 2a7 7 0 017 7c0 3.87-3.13 7-7 7s-7-3.13-7-7a7 7 0 017-7z"/><path d="M12 16v6"/><path d="M8 22h8"/></svg> Onboarding</h3>
                <p>Baseline mental health assessment</p>
            </a>
            <a class="card" href="/docs/mood?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/></svg> Mood Tracker</h3>
                <p>Daily mood entries with trends</p>
            </a>
            <a class="card" href="/docs/analytics?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg> Analytics</h3>
                <p>AI-powered mood analytics and insights</p>
            </a>
            <a class="card" href="/docs/reports?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> Reports</h3>
                <p>AI-generated mental health reports</p>
            </a>
            <a class="card" href="/docs/notes?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> Notes</h3>
                <p>Encrypted coping toolkit notes</p>
            </a>
            <a class="card" href="/docs/chats?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg> Chats</h3>
                <p>AI companion &amp; agentic chat sessions</p>
            </a>
            <a class="card" href="/docs/content?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4 12.5-12.5z"/></svg> Content</h3>
                <p>Admin blog/article management with cover images</p>
            </a>
            <a class="card" href="/docs/logs?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> Logs</h3>
                <p>Authentication &amp; activity audit logs</p>
            </a>
        </div>

        <h2>Developer Tools</h2>
        <div class="section-grid">
            <a class="card card-accent" href="/docs/playground?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polygon points="5 3 19 12 5 21 5 3"/></svg> API Playground</h3>
                <p>Interactive API tester — send requests and see live responses</p>
            </a>
            <a class="card card-accent" href="/docs/tests?password={pw}">
                <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg> Test Runner</h3>
                <p>Run the full test suite from the browser and see real output</p>
            </a>
        </div>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg> Authentication Flow</h2>
        <ol>
            <li><strong>GET</strong> <code>/api/auth/google/url</code> &rarr; redirect user to Google consent</li>
            <li>User consents &rarr; Google redirects with <code>code</code></li>
            <li><strong>POST</strong> <code>/api/auth/google/callback</code> with <code>{{"code":"..."}}</code> &rarr; returns JWT + user</li>
            <li>Use JWT as <code>Authorization: Bearer &lt;token&gt;</code> for all subsequent requests</li>
            <li>Verify email &rarr; complete onboarding &rarr; full access</li>
        </ol>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg> Access Levels</h2>
        <table>
            <tr><th>Extractor</th><th>Requires</th><th>Used By</th></tr>
            <tr><td><code>AuthUser</code></td><td>Valid JWT</td><td>GET /me, resend-verification, logs</td></tr>
            <tr><td><code>VerifiedUser</code></td><td>JWT + verified email</td><td>Onboarding, profile update</td></tr>
            <tr><td><code>FullUser</code></td><td>JWT + verified + onboarded</td><td>Mood, analytics, reports, notes, chats</td></tr>
            <tr><td><code>AdminUser</code></td><td>JWT + verified + admin role</td><td>Content management (create, update, delete)</td></tr>
        </table>

        <h2><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/></svg> Response Format</h2>
        <pre>{{"success": true, "data": {{ ... }} }}</pre>
        <pre>{{"success": false, "error": {{ "type": "BAD_REQUEST", "message": "..." }} }}</pre>

        <h2>Quick Start (JavaScript)</h2>
        <details open>
            <summary>Setup a reusable API client</summary>
            <pre><code class="lang-js">// api.js — reusable helper for all BSDY API calls
const BASE = '{base}';
let token = null; // set after login

async function api(path, options = {{}}) {{
  const headers = {{ 'Content-Type': 'application/json', ...options.headers }};
  if (token) headers['Authorization'] = `Bearer ${{token}}`;

  const res = await fetch(`${{BASE}}${{path}}`, {{ ...options, headers }});
  const data = await res.json();
  if (!res.ok) throw new Error(data.error?.message || res.statusText);
  return data;
}}

// Usage:
// const {{ url }} = await api('/api/auth/google/url');
// const {{ data }} = await api('/api/mood/today');</code></pre>
        </details>
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
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"// Get the Google sign-in URL and redirect the user
const res = await fetch('/api/auth/google/url');
const data = await res.json();
window.location.href = data.url;"#
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
      "role": "basic",
      "created_at": "2026-01-15 10:30:00"
    }},
    "is_new_user": true
  }}
}}"#,
                r#"// Exchange the auth code from Google redirect
const code = new URLSearchParams(window.location.search).get('code');

const res = await fetch('/api/auth/google/callback', {{
  method: 'POST',
  headers: {{ 'Content-Type': 'application/json' }},
  body: JSON.stringify({{ code }})
}});
const data = await res.json();

// Store the JWT token for subsequent requests
localStorage.setItem('token', data.data.token);
console.log('User:', data.data.user);"#
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
}}"#,
                r#"// Called when user clicks the link in verification email
const verifyToken = new URLSearchParams(window.location.search).get('token');
const res = await fetch(`/api/auth/verify-email?token=${{verifyToken}}`);
const data = await res.json();
console.log(data.message); // "Email verified successfully""#
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/auth/resend-verification', {{
  method: 'POST',
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.message);"#
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
    "role": "basic",
    "created_at": "2026-01-15 10:30:00"
  }}
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/auth/me', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const user = await res.json();
console.log(user.data.name);  // "John Doe"
console.log(user.data.role);  // "basic" or "admin""#
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/auth/me', {{
  method: 'PUT',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    name: 'John Updated',
    birth: '2000-05-15'
  }})
}});
const data = await res.json();
console.log(data.data);"#
            ),
        ].join("\n")
    );

    Html(render_page("Auth &mdash; BSDY Docs", &body)).into_response()
}

// ── Onboarding Docs ─────────────────────────────────────────

async fn docs_onboarding(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/onboarding/baseline', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    birth: '2000-05-15',
    family_background: 'No significant history',
    stress_level: 'moderate',
    anxiety_level: 'low',
    depression_level: 'low',
    sleep_quality: 'moderate',
    social_support: 'strong',
    coping_style: 'problem_focused',
    personality_traits: '["empathetic","introverted"]',
    mental_health_history: 'No prior diagnoses',
    current_medications: null,
    therapy_status: 'none',
    additional_notes: null
  }})
}});
const data = await res.json();
console.log('Risk level:', data.data.risk_level);"#
            ),
            endpoint(
                "GET",
                "/api/onboarding/baseline",
                "VerifiedUser",
                "Get the user's current baseline assessment (decrypted).",
                "// No request body",
                r#"{{ "success": true, "data": {{ ...baseline... }} }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/onboarding/baseline', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.data);"#
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
                r#"{{ "success": true, "data": {{ ...updated baseline... }} }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/onboarding/baseline', {{
  method: 'PUT',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    stress_level: 'high',
    therapy_status: 'active'
  }})
}});
const data = await res.json();
console.log(data.data);"#
            ),
        ].join("\n")
    );

    Html(render_page("Onboarding &mdash; BSDY Docs", &body)).into_response()
}

// ── Mood Docs ───────────────────────────────────────────────

async fn docs_mood(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/mood', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    mood_score: 7,
    energy_level: 6,
    anxiety_level: 3,
    stress_level: 4,
    sleep_hours: 7.5,
    sleep_quality: 7,
    appetite: 'normal',
    social_interaction: true,
    exercise_done: false,
    notes: 'Felt productive today',
    triggers: '["work deadline"]',
    activities: '["reading","walking"]'
  }})
}});
const data = await res.json();
console.log('Mood saved:', data.data.entry_date);"#
            ),
            endpoint(
                "GET",
                "/api/mood?from=2026-02-01&amp;to=2026-02-28&amp;limit=30",
                "FullUser",
                "Get mood entries for a date range. Defaults to last 30 days, max 90.",
                "// Query params: from, to (YYYY-MM-DD), limit",
                r#"{{
  "success": true,
  "data": [ {{ ...entry... }}, ... ],
  "count": 28
}}"#,
                r#"const token = localStorage.getItem('token');
const from = '2026-02-01';
const to   = '2026-02-28';
const res = await fetch(`/api/mood?from=${{from}}&to=${{to}}&limit=30`, {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(`Got ${{data.count}} entries`);
data.data.forEach(e => console.log(e.entry_date, e.mood_score));"#
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/mood/today', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
if (data.logged_today) {{
  console.log('Today mood score:', data.data.mood_score);
}} else {{
  console.log('No mood entry yet today');
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Mood &mdash; BSDY Docs", &body)).into_response()
}

// ── Analytics Docs ──────────────────────────────────────────

async fn docs_analytics(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/analytics/generate', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{ period_type: 'weekly' }})
}});
const data = await res.json();
console.log('Trend:', data.data.overall_mood_trend);
console.log('Avg score:', data.data.avg_mood_score);
console.log('Insights:', data.data.insights);"#
            ),
            endpoint(
                "GET",
                "/api/analytics?limit=10",
                "FullUser",
                "Get previously generated analytics summaries.",
                "// Query param: limit (default 10)",
                r#"{{ "success": true, "data": [ ... ], "count": 5 }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/analytics?limit=10', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
data.data.forEach(a => {{
  console.log(`${{a.period_type}} | ${{a.period_start}} - ${{a.period_end}} | ${{a.overall_mood_trend}}`);
}});"#
            ),
        ].join("\n")
    );

    Html(render_page("Analytics &mdash; BSDY Docs", &body)).into_response()
}

// ── Reports Docs ────────────────────────────────────────────

async fn docs_reports(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/reports/generate', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    report_type: 'weekly',
    period_start: null,
    period_end: null,
    send_email: true
  }})
}});
const data = await res.json();
console.log('Report:', data.data.title);
console.log('Status:', data.data.status);
console.log('Emailed:', data.data.sent_via_email);"#
            ),
            endpoint(
                "GET",
                "/api/reports?limit=10",
                "FullUser",
                "List generated reports.",
                "// Query param: limit (default 10)",
                r#"{{ "success": true, "data": [ ... ], "count": 3 }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/reports?limit=10', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
data.data.forEach(r => console.log(r.title, r.report_type, r.created_at));"#
            ),
            endpoint(
                "GET",
                "/api/reports/:report_id",
                "FullUser",
                "Get a specific report by ID.",
                "// Path param: report_id",
                r#"{{ "success": true, "data": {{ ...report... }} }}"#,
                r#"const token = localStorage.getItem('token');
const reportId = 'your-report-uuid';
const res = await fetch(`/api/reports/${{reportId}}`, {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.data.content);
console.log(data.data.ai_analysis);
console.log(data.data.recommendations);"#
            ),
        ].join("\n")
    );

    Html(render_page("Reports &mdash; BSDY Docs", &body)).into_response()
}

// ── Notes Docs ──────────────────────────────────────────────

async fn docs_notes(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/notes', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    title: 'Breathing exercise',
    content: '4-7-8 technique: breathe in 4s, hold 7s, out 8s',
    label: 'coping',
    is_pinned: true
  }})
}});
const data = await res.json();
console.log('Note created:', data.data.id);"#
            ),
            endpoint(
                "GET",
                "/api/notes?label=coping&amp;limit=50",
                "FullUser",
                "List notes. Optionally filter by label.",
                "// Query params: label (optional), limit (default 50)",
                r#"{{ "success": true, "data": [ ... ], "count": 12 }}"#,
                r#"const token = localStorage.getItem('token');
// All notes
const res = await fetch('/api/notes?limit=50', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
// Or filter by label
// const res = await fetch('/api/notes?label=coping&limit=50', {{ ... }});
const data = await res.json();
data.data.forEach(n => console.log(n.title, n.label, n.is_pinned));"#
            ),
            endpoint(
                "GET",
                "/api/notes/labels",
                "FullUser",
                "Get all distinct labels used by the user.",
                "// No request body",
                r#"{{ "success": true, "data": ["coping", "journal", "gratitude"] }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/notes/labels', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log('Labels:', data.data); // ["coping", "journal", "gratitude"]"#
            ),
            endpoint(
                "GET",
                "/api/notes/:note_id",
                "FullUser",
                "Get a specific note.",
                "// Path param: note_id",
                r#"{{ "success": true, "data": {{ ...note... }} }}"#,
                r#"const token = localStorage.getItem('token');
const noteId = 'your-note-uuid';
const res = await fetch(`/api/notes/${{noteId}}`, {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.data.title, data.data.content);"#
            ),
            endpoint(
                "PUT",
                "/api/notes/:note_id",
                "FullUser",
                "Update a note. Only provided fields are changed.",
                r#"{{ "title": "Updated title", "is_pinned": false }}"#,
                r#"{{ "success": true, "data": {{ ...updated note... }} }}"#,
                r#"const token = localStorage.getItem('token');
const noteId = 'your-note-uuid';
const res = await fetch(`/api/notes/${{noteId}}`, {{
  method: 'PUT',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{ title: 'Updated title', is_pinned: false }})
}});
const data = await res.json();
console.log('Updated:', data.data);"#
            ),
            endpoint(
                "DELETE",
                "/api/notes/:note_id",
                "FullUser",
                "Delete a note.",
                "// Path param: note_id",
                r#"{{ "success": true, "message": "Note deleted" }}"#,
                r#"const token = localStorage.getItem('token');
const noteId = 'your-note-uuid';
const res = await fetch(`/api/notes/${{noteId}}`, {{
  method: 'DELETE',
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.message); // "Note deleted""#
            ),
        ].join("\n")
    );

    Html(render_page("Notes &mdash; BSDY Docs", &body)).into_response()
}

// ── Chats Docs ──────────────────────────────────────────────

async fn docs_chats(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg> Chat</h1>
        <p>Two chat modes:</p>
        <ul>
            <li><strong>companion</strong> &mdash; Empathetic AI companion (default). Crisis-aware.</li>
            <li><strong>agentic</strong> &mdash; AI with tool access to mood data, analytics, notes, reports. Can create/edit notes and generate reports on demand.</li>
        </ul>

        <h2>Agentic Tools Reference</h2>
        <p>When using <code>agentic</code> mode the AI can call these tools automatically:</p>
        <table>
            <thead><tr><th>Tool</th><th>Description</th><th>Parameters</th></tr></thead>
            <tbody>
                <tr><td><code>GET_MOOD_ENTRIES</code></td><td>Retrieve mood tracker data</td><td>from, to (YYYY-MM-DD), limit</td></tr>
                <tr><td><code>GET_ANALYTICS</code></td><td>Get existing AI analytics summaries</td><td>limit</td></tr>
                <tr><td><code>GENERATE_ANALYTICS</code></td><td>Generate a new analytics summary</td><td>period (weekly|monthly|quarterly)</td></tr>
                <tr><td><code>GET_NOTES</code></td><td>Get user's coping toolkit notes</td><td>label (optional), limit</td></tr>
                <tr><td><code>CREATE_NOTE</code></td><td>Create a new coping toolkit note</td><td>title, content, label (optional), is_pinned (bool)</td></tr>
                <tr><td><code>UPDATE_NOTE</code></td><td>Edit an existing note</td><td>note_id (required), title, content, label, is_pinned</td></tr>
                <tr><td><code>DELETE_NOTE</code></td><td>Delete a note</td><td>note_id (required)</td></tr>
                <tr><td><code>GET_REPORTS</code></td><td>Get existing mental health reports</td><td>limit</td></tr>
                <tr><td><code>GENERATE_REPORT</code></td><td>Generate a new mental health report</td><td>report_type (weekly|monthly|yearly|custom), period_start, period_end, send_email</td></tr>
                <tr><td><code>SUGGEST_COPING_STRATEGIES</code></td><td>Analyze user data and suggest personalized coping strategies</td><td>context, save_as_notes (bool), label</td></tr>
                <tr><td><code>GET_BASELINE</code></td><td>Get user's baseline mental health assessment</td><td>(none)</td></tr>
            </tbody>
        </table>

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
}}"#,
                r#"const token = localStorage.getItem('token');

// Create a companion chat
const res = await fetch('/api/chats', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{ chat_type: 'companion' }})
  // or {{ chat_type: 'agentic' }} for tool-enabled chat
}});
const data = await res.json();
const chatId = data.data.id;
console.log('Chat created:', chatId);"#
            ),
            endpoint(
                "GET",
                "/api/chats?limit=20",
                "FullUser",
                "List chat sessions.",
                "// Query param: limit (default 20)",
                r#"{{ "success": true, "data": [ ... ], "count": 5 }}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/chats?limit=20', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
data.data.forEach(c => console.log(c.title, c.chat_type, c.message_count));"#
            ),
            endpoint(
                "GET",
                "/api/chats/:chat_id",
                "FullUser",
                "Get a specific chat.",
                "// Path param: chat_id",
                r#"{{ "success": true, "data": {{ ...chat... }} }}"#,
                r#"const token = localStorage.getItem('token');
const chatId = 'your-chat-uuid';
const res = await fetch(`/api/chats/${{chatId}}`, {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.data);"#
            ),
            endpoint(
                "PUT",
                "/api/chats/:chat_id",
                "FullUser",
                "Update chat title or active status.",
                r#"{{ "title": "My Session", "is_active": false }}"#,
                r#"{{ "success": true, "data": {{ ...updated chat... }} }}"#,
                r#"const token = localStorage.getItem('token');
const chatId = 'your-chat-uuid';
const res = await fetch(`/api/chats/${{chatId}}`, {{
  method: 'PUT',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{ title: 'My Session', is_active: false }})
}});
const data = await res.json();
console.log('Updated:', data.data);"#
            ),
            endpoint(
                "DELETE",
                "/api/chats/:chat_id",
                "FullUser",
                "Delete a chat and all its messages.",
                "// Path param: chat_id",
                r#"{{ "success": true, "message": "Chat deleted" }}"#,
                r#"const token = localStorage.getItem('token');
const chatId = 'your-chat-uuid';
const res = await fetch(`/api/chats/${{chatId}}`, {{
  method: 'DELETE',
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.message);"#
            ),
            endpoint(
                "GET",
                "/api/chats/:chat_id/messages?limit=50",
                "FullUser",
                "Get decrypted message history for a chat.",
                "// Query param: limit (default 50)",
                r#"{{ "success": true, "data": [ ... ], "count": 24 }}"#,
                r#"const token = localStorage.getItem('token');
const chatId = 'your-chat-uuid';
const res = await fetch(`/api/chats/${{chatId}}/messages?limit=50`, {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
data.data.forEach(m => {{
  console.log(`[${{m.role}}] ${{m.content}}`);
  if (m.has_tool_calls) console.log('Tools used:', m.tool_calls);
}});"#
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
}}"#,
                r#"const token = localStorage.getItem('token');
const chatId = 'your-chat-uuid';

const res = await fetch(`/api/chats/${{chatId}}/messages`, {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{ message: 'How am I doing this week?' }})
}});
const data = await res.json();
console.log('You:', data.data.user_message.content);
console.log('AI:', data.data.assistant_message.content);

// Check if the AI used tools (agentic mode)
if (data.data.assistant_message.has_tool_calls) {{
  console.log('Tools called:', data.data.assistant_message.tool_calls);
}}"#
            ),
        ].join("\n")
    );

    Html(render_page("Chat &mdash; BSDY Docs", &body)).into_response()
}

// ── Logs Docs ───────────────────────────────────────────────

async fn docs_logs(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> Audit Logs</h1>

        {blocks}
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "GET",
                "/api/logs/auth?page=1&amp;per_page=20",
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/logs/auth?page=1&per_page=20', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(`Page ${{data.data.page}} of ${{Math.ceil(data.data.total / data.data.per_page)}}`);
data.data.data.forEach(log => {{
  console.log(log.action, log.success, log.created_at);
}});"#
            ),
            endpoint(
                "GET",
                "/api/logs/activity?page=1&amp;per_page=20&amp;feature=mood_tracker",
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
}}"#,
                r#"const token = localStorage.getItem('token');
const res = await fetch('/api/logs/activity?page=1&per_page=20&feature=mood_tracker', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
data.data.data.forEach(log => {{
  console.log(`[${{log.feature}}] ${{log.action}} ${{log.entity_type}} ${{log.entity_id}}`);
}});"#
            ),
        ].join("\n")
    );

    Html(render_page("Logs &mdash; BSDY Docs", &body)).into_response()
}

// ── Content Docs ────────────────────────────────────────────

async fn docs_content(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4 12.5-12.5z"/></svg> Content Management</h1>
        <p class="subtitle">Admin-only blog/article CRUD with cover image uploads. Read access is public (published only).</p>

        <h2>User Roles</h2>
        <table>
            <tr><th>Role</th><th>Permissions</th></tr>
            <tr><td><code>basic</code></td><td>Read published content only (default for new users)</td></tr>
            <tr><td><code>admin</code></td><td>Full CRUD &mdash; create, update, delete content; upload cover images; view all statuses</td></tr>
        </table>

        <h2>Content Statuses</h2>
        <table>
            <tr><th>Status</th><th>Visibility</th><th>Description</th></tr>
            <tr><td><code>draft</code></td><td>Admin only</td><td>Work in progress, not visible to public</td></tr>
            <tr><td><code>published</code></td><td>Everyone</td><td>Live and publicly accessible</td></tr>
            <tr><td><code>archived</code></td><td>Admin only</td><td>Hidden from public, preserved for reference</td></tr>
        </table>

        <h2>Endpoints</h2>

        {blocks}

        <h2>Cover Image Upload</h2>
        <p>Images are stored on the server at <code>/uploads/content/</code> and served statically.</p>
        <ul>
            <li><strong>Max size:</strong> 10 MB</li>
            <li><strong>Allowed types:</strong> JPEG, PNG, WebP, GIF</li>
            <li><strong>Upload format:</strong> <code>multipart/form-data</code> with a single file field</li>
            <li><strong>URL format:</strong> <code>{{base_url}}/uploads/content/{{filename}}</code></li>
        </ul>
        "#,
        pw = pw,
        blocks = [
            endpoint(
                "GET",
                "/api/content?limit=20&amp;offset=0",
                "Public / AuthUser",
                "List content. Public users see published only. Admin sees all statuses.",
                "// Query params: limit (default 20, max 100), offset (default 0)",
                r#"{{
  "success": true,
  "data": [
    {{
      "id": "uuid",
      "author_id": "uuid",
      "title": "Understanding Anxiety",
      "slug": "understanding-anxiety",
      "excerpt": "A brief guide...",
      "cover_image_url": "http://localhost:8000/uploads/content/img.jpg",
      "status": "published",
      "published_at": "2026-03-01 09:00:00",
      "created_at": "2026-02-28 12:00:00"
    }}
  ],
  "total": 5,
  "limit": 20,
  "offset": 0
}}"#,
                r#"// Public access (no auth needed for published content)
const res = await fetch('/api/content?limit=20&offset=0');
const data = await res.json();
data.data.forEach(c => {{
  console.log(c.title, c.slug, c.status);
}});

// Admin access (sees all statuses)
const token = localStorage.getItem('token');
const adminRes = await fetch('/api/content?limit=20&offset=0', {{
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const adminData = await adminRes.json();
console.log('Total content:', adminData.total);"#
            ),
            endpoint(
                "GET",
                "/api/content/:content_id",
                "Public / AuthUser",
                "Get full content by ID. Public users can only access published content.",
                "// No request body",
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "author_id": "uuid",
    "title": "Understanding Anxiety",
    "slug": "understanding-anxiety",
    "body": "Full article content in HTML or Markdown...",
    "excerpt": "A brief guide...",
    "cover_image_url": "http://localhost:8000/uploads/content/img.jpg",
    "status": "published",
    "published_at": "2026-03-01 09:00:00",
    "created_at": "2026-02-28 12:00:00",
    "updated_at": "2026-03-01 10:00:00"
  }}
}}"#,
                r#"const contentId = 'your-content-uuid';
const res = await fetch(`/api/content/${{contentId}}`);
const data = await res.json();
console.log(data.data.title);
console.log(data.data.body);

// Render in HTML
document.querySelector('#article').innerHTML = data.data.body;
document.querySelector('#cover').src = data.data.cover_image_url;"#
            ),
            endpoint(
                "GET",
                "/api/content/slug/:slug",
                "Public / AuthUser",
                "Get full content by slug. Useful for SEO-friendly URLs.",
                "// Example: /api/content/slug/understanding-anxiety",
                r#"{{
  "success": true,
  "data": {{ "id": "...", "title": "...", "slug": "understanding-anxiety", ... }}
}}"#,
                r#"// Great for building SEO-friendly blog pages
const slug = 'understanding-anxiety'; // from URL path
const res = await fetch(`/api/content/slug/${{slug}}`);
const data = await res.json();
document.title = data.data.title;
document.querySelector('#article').innerHTML = data.data.body;"#
            ),
            endpoint(
                "POST",
                "/api/content",
                "AdminUser",
                "Create a new content entry. Only admin users can create content.",
                r#"{{
  "title": "Understanding Anxiety",
  "body": "Full article content...",
  "excerpt": "A brief guide to managing anxiety",
  "status": "draft"
}}"#,
                r#"{{
  "success": true,
  "data": {{
    "id": "uuid",
    "author_id": "admin-user-id",
    "title": "Understanding Anxiety",
    "slug": "understanding-anxiety",
    "body": "Full article content...",
    "excerpt": "A brief guide to managing anxiety",
    "cover_image_url": null,
    "status": "draft",
    "published_at": null,
    "created_at": "...",
    "updated_at": "..."
  }}
}}"#,
                r#"const token = localStorage.getItem('token'); // must be admin
const res = await fetch('/api/content', {{
  method: 'POST',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    title: 'Understanding Anxiety',
    body: 'Full article content...',
    excerpt: 'A brief guide to managing anxiety',
    status: 'draft' // 'draft' | 'published' | 'archived'
  }})
}});
const data = await res.json();
console.log('Created:', data.data.slug);"#
            ),
            endpoint(
                "PUT",
                "/api/content/:content_id",
                "AdminUser",
                "Update an existing content entry. All fields are optional.",
                r#"{{
  "title": "Updated Title",
  "body": "Updated body content...",
  "status": "published"
}}"#,
                r#"{{
  "success": true,
  "data": {{ "id": "...", "status": "published", "published_at": "2026-03-01 09:00:00", ... }}
}}"#,
                r#"const token = localStorage.getItem('token'); // must be admin
const contentId = 'your-content-uuid';
const res = await fetch(`/api/content/${{contentId}}`, {{
  method: 'PUT',
  headers: {{
    'Authorization': `Bearer ${{token}}`,
    'Content-Type': 'application/json'
  }},
  body: JSON.stringify({{
    title: 'Updated Title',
    status: 'published' // setting to 'published' auto-sets published_at
  }})
}});
const data = await res.json();
console.log('Published at:', data.data.published_at);"#
            ),
            endpoint(
                "DELETE",
                "/api/content/:content_id",
                "AdminUser",
                "Delete a content entry and its cover image from the server.",
                "// No request body",
                r#"{{
  "success": true,
  "message": "Content deleted"
}}"#,
                r#"const token = localStorage.getItem('token'); // must be admin
const contentId = 'your-content-uuid';
const res = await fetch(`/api/content/${{contentId}}`, {{
  method: 'DELETE',
  headers: {{ 'Authorization': `Bearer ${{token}}` }}
}});
const data = await res.json();
console.log(data.message); // "Content deleted""#
            ),
            endpoint(
                "POST",
                "/api/content/:content_id/cover",
                "AdminUser",
                "Upload or replace the cover image. Send as multipart/form-data.",
                "// Content-Type: multipart/form-data\n// Field: file (JPEG, PNG, WebP, or GIF, max 10MB)",
                r#"{{
  "success": true,
  "data": {{
    "id": "...",
    "cover_image_url": "http://localhost:8000/uploads/content/uuid_uuid.jpg",
    ...
  }}
}}"#,
                r#"const token = localStorage.getItem('token'); // must be admin
const contentId = 'your-content-uuid';
const fileInput = document.querySelector('input[type="file"]');

const formData = new FormData();
formData.append('file', fileInput.files[0]);

const res = await fetch(`/api/content/${{contentId}}/cover`, {{
  method: 'POST',
  headers: {{ 'Authorization': `Bearer ${{token}}` }},
  // Do NOT set Content-Type — browser sets it with boundary
  body: formData
}});
const data = await res.json();
console.log('Cover URL:', data.data.cover_image_url);"#
            ),
        ].join("\n")
    );

    Html(render_page("Content &mdash; BSDY Docs", &body)).into_response()
}

// ── API Playground ──────────────────────────────────────────

async fn docs_playground(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><polygon points="5 3 19 12 5 21 5 3"/></svg> API Playground</h1>
        <p class="subtitle">Send live requests to the API and inspect responses in real time.</p>

        <div class="playground">
            <div class="pg-row">
                <select id="pg-method">
                    <option value="GET">GET</option>
                    <option value="POST" selected>POST</option>
                    <option value="PUT">PUT</option>
                    <option value="DELETE">DELETE</option>
                </select>
                <input id="pg-url" type="text" placeholder="/api/auth/google/url" value="/api/auth/google/url" />
                <button id="pg-send" onclick="pgSend()">Send</button>
            </div>

            <div class="pg-section">
                <h4>Headers</h4>
                <div id="pg-headers">
                    <div class="pg-header-row">
                        <input type="text" placeholder="Key" value="Content-Type" class="pg-hdr-key" />
                        <input type="text" placeholder="Value" value="application/json" class="pg-hdr-val" />
                        <button onclick="this.parentElement.remove()" class="pg-remove">&times;</button>
                    </div>
                    <div class="pg-header-row">
                        <input type="text" placeholder="Key" value="Authorization" class="pg-hdr-key" />
                        <input type="text" placeholder="Value" value="Bearer " class="pg-hdr-val" />
                        <button onclick="this.parentElement.remove()" class="pg-remove">&times;</button>
                    </div>
                </div>
                <button onclick="addHeader()" class="pg-add-btn">+ Add Header</button>
            </div>

            <div class="pg-section">
                <h4>Request Body <span style="color:var(--text-muted);font-weight:normal">(JSON)</span></h4>
                <textarea id="pg-body" rows="8" placeholder='{{"key": "value"}}'></textarea>
            </div>

            <div class="pg-section">
                <h4>Response</h4>
                <div id="pg-status" style="display:none;margin-bottom:0.5rem;"></div>
                <div id="pg-timing" style="display:none;margin-bottom:0.5rem;color:var(--text-muted);font-size:0.85em;"></div>
                <pre id="pg-response" style="min-height:100px;">// Click "Send" to execute the request</pre>
            </div>

            <div class="pg-section">
                <h4>Quick Fill Presets</h4>
                <div class="pg-presets">
                    <button onclick="pgPreset('GET','/health','','')">Health Check</button>
                    <button onclick="pgPreset('GET','/api/auth/google/url','','')">Get OAuth URL</button>
                    <button onclick="pgPreset('GET','/api/auth/me','','')">Get Profile</button>
                    <button onclick="pgPreset('POST','/api/mood','','{pg_mood}')">Log Mood</button>
                    <button onclick="pgPreset('GET','/api/mood/today','','')">Today&apos;s Mood</button>
                    <button onclick="pgPreset('GET','/api/notes?limit=10','','')">List Notes</button>
                    <button onclick="pgPreset('POST','/api/chats','','{pg_chats}')">Create Chat</button>
                    <button onclick="pgPreset('GET','/api/content?limit=10','','')">List Content</button>
                    <button onclick="pgPreset('POST','/api/analytics/generate','','{pg_analytics}')">Generate Analytics</button>
                    <button onclick="pgPreset('POST','/api/reports/generate','','{pg_reports}')">Generate Report</button>
                </div>
            </div>
        </div>

        <script>
        function addHeader() {{
            const div = document.createElement('div');
            div.className = 'pg-header-row';
            div.innerHTML = '<input type="text" placeholder="Key" class="pg-hdr-key"/><input type="text" placeholder="Value" class="pg-hdr-val"/><button onclick="this.parentElement.remove()" class="pg-remove">&times;</button>';
            document.getElementById('pg-headers').appendChild(div);
        }}

        function pgPreset(method, url, hdrs, body) {{
            document.getElementById('pg-method').value = method;
            document.getElementById('pg-url').value = url;
            document.getElementById('pg-body').value = body;
        }}

        async function pgSend() {{
            const btn = document.getElementById('pg-send');
            const method = document.getElementById('pg-method').value;
            const url = document.getElementById('pg-url').value;
            const bodyText = document.getElementById('pg-body').value.trim();
            const statusEl = document.getElementById('pg-status');
            const timingEl = document.getElementById('pg-timing');
            const responseEl = document.getElementById('pg-response');

            btn.disabled = true;
            btn.textContent = 'Sending...';
            statusEl.style.display = 'none';
            timingEl.style.display = 'none';
            responseEl.textContent = '// Loading...';

            const headers = {{}};
            document.querySelectorAll('.pg-header-row').forEach(row => {{
                const k = row.querySelector('.pg-hdr-key')?.value?.trim();
                const v = row.querySelector('.pg-hdr-val')?.value?.trim();
                if (k && v) headers[k] = v;
            }});

            const opts = {{ method, headers }};
            if (bodyText && method !== 'GET') {{
                opts.body = bodyText;
            }}

            const start = performance.now();
            try {{
                const res = await fetch(url, opts);
                const elapsed = Math.round(performance.now() - start);

                statusEl.style.display = 'block';
                statusEl.innerHTML = `<strong>Status:</strong> <span style="color:${{res.ok?'var(--green)':'var(--red)'}}">${{res.status}} ${{res.statusText}}</span>`;

                timingEl.style.display = 'block';
                timingEl.textContent = `Time: ${{elapsed}}ms`;

                const ct = res.headers.get('content-type') || '';
                if (ct.includes('json')) {{
                    const json = await res.json();
                    responseEl.textContent = JSON.stringify(json, null, 2);
                }} else {{
                    responseEl.textContent = await res.text();
                }}
            }} catch(e) {{
                statusEl.style.display = 'block';
                statusEl.innerHTML = '<strong>Error:</strong> <span style="color:var(--red)">' + e.message + '</span>';
                responseEl.textContent = e.stack || e.toString();
            }}

            btn.disabled = false;
            btn.textContent = 'Send';
        }}
        </script>
        "#,
        pw = pw,
        pg_mood = r#"{\n  \"mood_score\": 7,\n  \"energy_level\": 6,\n  \"anxiety_level\": 3,\n  \"stress_level\": 4,\n  \"sleep_hours\": 7.5,\n  \"sleep_quality\": 7,\n  \"appetite\": \"normal\",\n  \"social_interaction\": true,\n  \"exercise_done\": false,\n  \"notes\": \"Felt productive today\"\n}"#,
        pg_chats = r#"{\n  \"chat_type\": \"companion\"\n}"#,
        pg_analytics = r#"{\n  \"period_type\": \"weekly\"\n}"#,
        pg_reports = r#"{\n  \"report_type\": \"weekly\",\n  \"send_email\": false\n}"#
    );

    Html(render_page("API Playground &mdash; BSDY Docs", &body)).into_response()
}

// ── Test Runner ─────────────────────────────────────────────

async fn docs_tests(State(state): State<AppState>, Query(q): Query<DocsQuery>) -> Response {
    if let Some(r) = check_password(&state, &q) {
        return r;
    }
    let pw = q.password.as_deref().unwrap_or("");

    let body = format!(
        r#"
        <p><a href="/docs?password={pw}">&larr; Back to Index</a></p>
        <h1><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(88,166,255)" stroke-width="2" style="vertical-align:middle;margin-right:6px"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg> Test Runner</h1>
        <p class="subtitle">Execute the test suite directly from the browser. Results are displayed in real time.</p>

        <div class="test-runner">
            <div class="tr-controls">
                <button id="tr-run-unit" onclick="runTests('unit')">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                    Run Unit Tests
                </button>
                <button id="tr-run-all" onclick="runTests('all')">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                    Run All Tests (incl. integration)
                </button>
                <span id="tr-status" class="tr-badge" style="display:none;"></span>
            </div>

            <h3>Test Files</h3>
            <table class="tr-info">
                <tr><th>File</th><th>Coverage</th><th>Requires</th></tr>
                <tr><td><code>test_config_crypto</code></td><td>Config parsing, env defaults, AES encrypt/decrypt</td><td>Nothing</td></tr>
                <tr><td><code>test_errors_models</code></td><td>Error types, model serialization, content/role/agent tests</td><td>Nothing</td></tr>
                <tr><td><code>test_auth_chat</code></td><td>JWT, auth service, chat models, slug generation</td><td>Nothing</td></tr>
                <tr><td><code>test_database</code></td><td>DB pool, migrations, CRUD queries</td><td>MariaDB</td></tr>
                <tr><td><code>test_gemini</code></td><td>Request/response parsing (unit), live AI calls (ignored)</td><td>Gemini API key</td></tr>
                <tr><td><code>test_email</code></td><td>Template rendering (unit), live SMTP send (ignored)</td><td>Brevo credentials</td></tr>
                <tr><td><code>test_services</code></td><td>Service-layer integration (mood, analytics, content CRUD)</td><td>MariaDB</td></tr>
                <tr><td><code>test_routes</code></td><td>HTTP endpoints, auth guards, content routes, scheduler</td><td>MariaDB</td></tr>
            </table>

            <h3>Output</h3>
            <pre id="tr-output" class="tr-terminal">// Click a button above to run tests.
// Unit tests require no external services.
// Integration tests require MariaDB + .env configured.</pre>
        </div>

        <script>
        async function runTests(mode) {{
            const btnUnit = document.getElementById('tr-run-unit');
            const btnAll  = document.getElementById('tr-run-all');
            const status  = document.getElementById('tr-status');
            const output  = document.getElementById('tr-output');

            btnUnit.disabled = true;
            btnAll.disabled = true;
            status.style.display = 'inline-block';
            status.className = 'tr-badge running';
            status.textContent = 'Running...';
            output.textContent = `$ cargo test ${{mode === 'all' ? '-- --include-ignored' : ''}}\n\nPlease wait, compiling and running tests...\n`;

            try {{
                const res = await fetch(`/docs/run-tests?password={pw}&mode=${{mode}}`, {{
                    method: 'POST'
                }});
                const data = await res.json();

                if (data.success) {{
                    const passed = data.exit_code === 0;
                    status.className = passed ? 'tr-badge passed' : 'tr-badge failed';
                    status.textContent = passed ? 'PASSED' : 'FAILED';

                    let text = '';
                    if (data.stdout) text += data.stdout;
                    if (data.stderr) text += '\n--- stderr ---\n' + data.stderr;
                    output.textContent = text || '(no output)';
                }} else {{
                    status.className = 'tr-badge failed';
                    status.textContent = 'ERROR';
                    output.textContent = data.error || 'Unknown error';
                }}
            }} catch(e) {{
                status.className = 'tr-badge failed';
                status.textContent = 'ERROR';
                output.textContent = 'Network error: ' + e.message;
            }}

            btnUnit.disabled = false;
            btnAll.disabled = false;
        }}
        </script>
        "#,
        pw = pw
    );

    Html(render_page("Test Runner &mdash; BSDY Docs", &body)).into_response()
}

// ── Run Tests Handler (POST) ────────────────────────────────

#[derive(Debug, Deserialize)]
struct TestQuery {
    password: Option<String>,
    mode: Option<String>,
}

async fn run_tests_handler(State(state): State<AppState>, Query(q): Query<TestQuery>) -> Response {
    // Password check
    match &q.password {
        Some(p) if p == &state.config.docs.password => {}
        _ => {
            return Json(
                serde_json::json!({
                "success": false,
                "error": "Unauthorized"
            })
            ).into_response();
        }
    }

    let mode = q.mode.as_deref().unwrap_or("unit");

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("test");

    if mode == "all" {
        cmd.args(["--", "--include-ignored"]);
    }

    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));

    match cmd.output().await {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Json(
                serde_json::json!({
                "success": true,
                "exit_code": out.status.code(),
                "stdout": stdout,
                "stderr": stderr,
            })
            ).into_response()
        }
        Err(e) => {
            Json(
                serde_json::json!({
                "success": false,
                "error": format!("Failed to run tests: {}", e)
            })
            ).into_response()
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────

fn endpoint(
    method: &str,
    path: &str,
    auth: &str,
    description: &str,
    request_example: &str,
    response_example: &str,
    js_code: &str
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
            <details class="js-example">
                <summary>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;margin-right:4px"><path d="M20 3H4a1 1 0 00-1 1v16a1 1 0 001 1h16a1 1 0 001-1V4a1 1 0 00-1-1z"/><path d="M7.5 15.5l3-3-3-3"/><line x1="13" y1="15.5" x2="17" y2="15.5"/></svg>
                    JavaScript Example
                </summary>
                <pre><code>{js_code}</code></pre>
            </details>
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
        h3 {{ margin-top: 1.5rem; margin-bottom: 0.5rem; }}
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
        pre code {{ background: transparent; padding: 0; }}
        table {{ width: 100%; border-collapse: collapse; margin: 1rem 0; }}
        th, td {{ text-align: left; padding: 0.5rem 0.75rem; border: 1px solid var(--border); }}
        th {{ background: var(--surface); }}
        ol, ul {{ padding-left: 1.5rem; margin: 0.5rem 0; }}
        li {{ margin: 0.3rem 0; }}

        /* Cards */
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
        .card h3 {{ margin-bottom: 0.3rem; margin-top: 0; }}
        .card p {{ color: var(--text-muted); font-size: 0.9em; }}
        .card-accent {{ border-color: var(--purple); }}
        .card-accent:hover {{ border-color: var(--green); }}

        /* Endpoint blocks */
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

        /* JS Example (collapsible) */
        .js-example {{
            margin-top: 0.75rem;
            border: 1px solid var(--border);
            border-radius: 6px;
            overflow: hidden;
        }}
        .js-example summary {{
            cursor: pointer;
            padding: 0.5rem 0.75rem;
            background: rgba(88,166,255,0.08);
            color: var(--accent);
            font-size: 0.85em;
            font-weight: 600;
            user-select: none;
        }}
        .js-example summary:hover {{ background: rgba(88,166,255,0.15); }}
        .js-example pre {{
            margin: 0;
            border: none;
            border-radius: 0;
            border-top: 1px solid var(--border);
        }}

        /* Error card */
        .error-card {{
            background: var(--surface);
            border: 1px solid var(--red);
            border-radius: 8px;
            padding: 2rem;
            text-align: center;
            margin-top: 4rem;
        }}
        .error-card h2 {{ color: var(--red); border: none; }}

        /* Playground */
        .playground {{ margin-top: 1rem; }}
        .pg-row {{
            display: flex;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }}
        .pg-row select {{
            background: var(--surface);
            color: var(--text);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 0.5rem 0.75rem;
            font-size: 0.9em;
            font-weight: 700;
        }}
        .pg-row input {{
            flex: 1;
            background: var(--surface);
            color: var(--text);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 0.5rem 0.75rem;
            font-family: monospace;
            font-size: 0.9em;
        }}
        .pg-row button, .pg-add-btn, .tr-controls button, .pg-presets button {{
            background: var(--accent);
            color: #fff;
            border: none;
            border-radius: 6px;
            padding: 0.5rem 1.25rem;
            font-weight: 600;
            cursor: pointer;
            font-size: 0.9em;
        }}
        .pg-row button:hover, .pg-add-btn:hover, .tr-controls button:hover, .pg-presets button:hover {{ opacity: 0.85; }}
        .pg-row button:disabled, .tr-controls button:disabled {{ opacity: 0.4; cursor: not-allowed; }}
        .pg-section {{ margin-bottom: 1rem; }}
        .pg-section h4 {{ margin-bottom: 0.4rem; font-size: 0.9em; color: var(--text-muted); }}
        .pg-header-row {{
            display: flex;
            gap: 0.4rem;
            margin-bottom: 0.3rem;
        }}
        .pg-header-row input {{
            flex: 1;
            background: var(--surface);
            color: var(--text);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 0.35rem 0.5rem;
            font-family: monospace;
            font-size: 0.85em;
        }}
        .pg-remove {{
            background: var(--red);
            color: #fff;
            border: none;
            border-radius: 4px;
            padding: 0.35rem 0.6rem;
            cursor: pointer;
            font-size: 0.85em;
        }}
        .pg-add-btn {{
            font-size: 0.8em;
            padding: 0.3rem 0.8rem;
            background: var(--surface);
            border: 1px solid var(--border);
            color: var(--accent);
            margin-top: 0.3rem;
        }}
        #pg-body {{
            width: 100%;
            background: var(--surface);
            color: var(--text);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 0.75rem;
            font-family: monospace;
            font-size: 0.85em;
            resize: vertical;
        }}
        .pg-presets {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.4rem;
        }}
        .pg-presets button {{
            font-size: 0.75em;
            padding: 0.3rem 0.7rem;
            background: var(--surface);
            border: 1px solid var(--border);
            color: var(--text);
        }}

        /* Test Runner */
        .test-runner {{ margin-top: 1rem; }}
        .tr-controls {{
            display: flex;
            gap: 0.5rem;
            align-items: center;
            flex-wrap: wrap;
            margin-bottom: 1rem;
        }}
        .tr-controls button {{
            background: var(--green);
        }}
        .tr-badge {{
            font-size: 0.8em;
            padding: 0.2rem 0.8rem;
            border-radius: 10px;
            font-weight: 700;
        }}
        .tr-badge.running {{ background: var(--yellow); color: #000; }}
        .tr-badge.passed {{ background: var(--green); color: #fff; }}
        .tr-badge.failed {{ background: var(--red); color: #fff; }}
        .tr-terminal {{
            min-height: 200px;
            max-height: 600px;
            overflow: auto;
            font-size: 0.8em;
            line-height: 1.4;
            color: var(--green);
            background: #0a0e14;
            border-color: var(--border);
        }}
        .tr-info {{ font-size: 0.9em; }}
    </style>
</head>
<body>
    {body}
</body>
</html>"#
    )
}
