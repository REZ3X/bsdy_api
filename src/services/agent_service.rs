use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::{
    crypto::CryptoService,
    error::{AppError, Result},
    models::chat::{
        AgentResponse, ChatMessageResponse, ChatMessageRow, ChatRow, SendMessageRequest, ToolCall,
        ToolCallRequest, ToolResult,
    },
    services::{
        analytics_service::AnalyticsService, chat_service::ChatService,
        email_service::EmailService, gemini_service::GeminiService, note_service::NoteService,
        report_service::ReportService,
    },
};

pub struct AgentService;

impl AgentService {
    /// Process a message in an agentic chat — the AI can call tools to access
    /// mood tracker data, analytics, reports, and notes.
    pub async fn process_message(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email_service: &EmailService,
        user_id: &str,
        user_name: &str,
        user_email: &str,
        chat_id: &str,
        encryption_salt: &str,
        req: &SendMessageRequest,
    ) -> Result<(ChatMessageResponse, ChatMessageResponse)> {
        let chat: ChatRow = sqlx::query_as(
            "SELECT * FROM chats WHERE id = ? AND user_id = ? AND chat_type = 'agentic'",
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Agentic chat not found".into()))?;

        let severity = ChatService::detect_severity(&req.message);
        let user_msg = ChatService::save_message(
            pool,
            crypto,
            chat_id,
            user_id,
            "user",
            &req.message,
            None,
            None,
            &severity,
            encryption_salt,
        )
        .await?;

        let history_rows: Vec<ChatMessageRow> = sqlx::query_as(
            r#"SELECT * FROM chat_messages
               WHERE chat_id = ? AND user_id = ?
               ORDER BY created_at DESC LIMIT 16"#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let mut history: Vec<(String, String)> = history_rows
            .iter()
            .rev()
            .filter_map(|row| {
                let content = crypto.decrypt(&row.content_enc, encryption_salt).ok()?;
                let role = if row.role == "user" { "user" } else { "model" };
                Some((role.to_string(), content))
            })
            .collect();

        if history.last().map(|(r, _)| r.as_str()) == Some("user") {
            history.pop();
        }

        let system_prompt = Self::build_agent_system_prompt(user_name);

        let ai_response = gemini
            .generate_chat_response(&system_prompt, &history, &req.message, 1.0)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let (response_text, tool_calls, tool_results) = Self::parse_and_execute_tools(
            pool,
            crypto,
            gemini,
            email_service,
            user_id,
            user_name,
            user_email,
            encryption_salt,
            &ai_response,
        )
        .await?;

        let final_response = if !tool_results.is_empty() {
            let tool_context: String = tool_results
                .iter()
                .map(|tr| {
                    format!(
                        "Tool: {}\nSuccess: {}\nResult: {}",
                        tr.tool_name,
                        tr.success,
                        serde_json::to_string_pretty(&tr.result).unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            let follow_up = format!(
                "{}\n\nTOOL RESULTS:\n{}\n\nProvide a natural, conversational summary of the tool results to the user. Be friendly and helpful.",
                response_text,
                tool_context
            );

            gemini
                .generate_with_system(
                    &follow_up,
                    Some(
                        "You are BSDY, a helpful mental health AI assistant with access to user data. Summarize tool results naturally."
                    ),
                    0.7,
                    4096
                ).await
                .unwrap_or(response_text)
        } else {
            response_text
        };

        let tc_json = if !tool_calls.is_empty() {
            Some(serde_json::to_value(&tool_calls).unwrap_or(Value::Null))
        } else {
            None
        };
        let tr_json = if !tool_results.is_empty() {
            Some(serde_json::to_value(&tool_results).unwrap_or(Value::Null))
        } else {
            None
        };

        let assistant_msg = ChatService::save_message(
            pool,
            crypto,
            chat_id,
            user_id,
            "assistant",
            &final_response,
            tc_json.as_ref(),
            tr_json.as_ref(),
            "none",
            encryption_salt,
        )
        .await?;

        let new_count = chat.message_count + 2;
        sqlx::query("UPDATE chats SET message_count = ?, updated_at = NOW() WHERE id = ?")
            .bind(new_count)
            .bind(chat_id)
            .execute(pool)
            .await
            .ok();

        if chat.message_count == 0 {
            if let Ok(title) = gemini.generate_chat_title(&req.message).await {
                sqlx::query("UPDATE chats SET title = ? WHERE id = ?")
                    .bind(&title)
                    .bind(chat_id)
                    .execute(pool)
                    .await
                    .ok();
            }
        }

        Ok((user_msg, assistant_msg))
    }

    async fn parse_and_execute_tools(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email_service: &EmailService,
        user_id: &str,
        user_name: &str,
        user_email: &str,
        encryption_salt: &str,
        ai_response: &str,
    ) -> Result<(String, Vec<ToolCall>, Vec<ToolResult>)> {
        let parsed = serde_json::from_str::<AgentResponse>(ai_response.trim());

        let parsed = parsed.or_else(|_| {
            let cleaned = ai_response
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            serde_json::from_str::<AgentResponse>(cleaned)
        });

        if let Ok(agent_resp) = parsed {
            if !agent_resp.tool_calls.is_empty() {
                let mut tool_calls = Vec::new();
                let mut tool_results = Vec::new();

                for tc in &agent_resp.tool_calls {
                    tracing::info!("Executing agentic tool: {}", tc.tool_name);

                    let result = Self::execute_tool(
                        pool,
                        crypto,
                        gemini,
                        email_service,
                        user_id,
                        user_name,
                        user_email,
                        encryption_salt,
                        tc,
                    )
                    .await;

                    let (success, result_value) = match result {
                        Ok(v) => (true, v),
                        Err(e) => {
                            tracing::error!("Tool {} failed: {:?}", tc.tool_name, e);
                            (false, json!({ "error": format!("{}", e) }))
                        }
                    };

                    tool_calls.push(ToolCall {
                        tool_name: tc.tool_name.clone(),
                        parameters: tc.parameters.clone(),
                    });
                    tool_results.push(ToolResult {
                        tool_name: tc.tool_name.clone(),
                        result: result_value,
                        success,
                    });
                }

                return Ok((agent_resp.response, tool_calls, tool_results));
            }
            return Ok((agent_resp.response, vec![], vec![]));
        }

        Ok((ai_response.to_string(), vec![], vec![]))
    }

    async fn execute_tool(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email_service: &EmailService,
        user_id: &str,
        user_name: &str,
        user_email: &str,
        encryption_salt: &str,
        tool_call: &ToolCallRequest,
    ) -> std::result::Result<Value, AppError> {
        match tool_call.tool_name.to_uppercase().as_str() {
            "GET_MOOD_ENTRIES" | "GET_MOOD_LOGS" => {
                Self::tool_get_mood_entries(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GET_ANALYTICS" | "GET_ANALYTICS_SUMMARY" => {
                Self::tool_get_analytics(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GENERATE_ANALYTICS" => {
                Self::tool_generate_analytics(
                    pool,
                    crypto,
                    gemini,
                    user_id,
                    user_name,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GET_NOTES" | "GET_COPING_NOTES" => {
                Self::tool_get_notes(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GET_REPORTS" => {
                Self::tool_get_reports(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GENERATE_REPORT" => {
                Self::tool_generate_report(
                    pool,
                    crypto,
                    gemini,
                    email_service,
                    user_id,
                    user_name,
                    user_email,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "GET_BASELINE" | "GET_MENTAL_PROFILE" => {
                Self::tool_get_baseline(pool, crypto, user_id, encryption_salt).await
            }
            "CREATE_NOTE" | "CREATE_COPING_NOTE" => {
                Self::tool_create_note(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "UPDATE_NOTE" | "EDIT_NOTE" => {
                Self::tool_update_note(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            "DELETE_NOTE" | "REMOVE_NOTE" => {
                Self::tool_delete_note(pool, user_id, &tool_call.parameters).await
            }
            "SUGGEST_COPING_STRATEGIES" | "SUGGEST_COPING" => {
                Self::tool_suggest_coping_strategies(
                    pool,
                    crypto,
                    gemini,
                    user_id,
                    user_name,
                    encryption_salt,
                    &tool_call.parameters,
                )
                .await
            }
            _ => Err(AppError::BadRequest(format!(
                "Unknown tool: {}",
                tool_call.tool_name
            ))),
        }
    }

    // ── Tool Implementations ────────────────────────────────

    async fn tool_get_mood_entries(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let limit = params["limit"].as_i64().unwrap_or(14) as u32;

        let from = params["from"]
            .as_str()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let to = params["to"]
            .as_str()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        let entries = crate::services::mood_service::MoodService::get_mood_entries(
            pool,
            crypto,
            user_id,
            encryption_salt,
            from,
            to,
            Some(limit),
        )
        .await?;

        Ok(json!({
            "success": true,
            "entries": entries,
            "count": entries.len()
        }))
    }

    async fn tool_get_analytics(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let limit = params["limit"].as_i64().unwrap_or(5);

        let summaries =
            AnalyticsService::get_summaries(pool, crypto, user_id, encryption_salt, limit).await?;

        Ok(json!({
            "success": true,
            "summaries": summaries,
            "count": summaries.len()
        }))
    }

    async fn tool_generate_analytics(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        user_id: &str,
        user_name: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let period = params["period"].as_str().unwrap_or("weekly");

        let summary = AnalyticsService::generate_summary(
            pool,
            crypto,
            gemini,
            user_id,
            user_name,
            encryption_salt,
            period,
            "agentic",
        )
        .await?;

        Ok(serde_json::to_value(&summary).unwrap_or(json!({"success": true})))
    }

    async fn tool_get_notes(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let label = params["label"].as_str();
        let limit = params["limit"].as_i64().unwrap_or(20);

        let notes =
            NoteService::get_notes(pool, crypto, user_id, encryption_salt, label, limit).await?;

        Ok(json!({
            "success": true,
            "notes": notes,
            "count": notes.len()
        }))
    }

    async fn tool_get_reports(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let limit = params["limit"].as_i64().unwrap_or(5);

        let reports =
            ReportService::get_reports(pool, crypto, user_id, encryption_salt, limit).await?;

        Ok(json!({
            "success": true,
            "reports": reports,
            "count": reports.len()
        }))
    }

    async fn tool_generate_report(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email_service: &EmailService,
        user_id: &str,
        user_name: &str,
        user_email: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let req = crate::models::mental::GenerateReportRequest {
            report_type: params["report_type"].as_str().map(String::from),
            period_start: params["period_start"].as_str().map(String::from),
            period_end: params["period_end"].as_str().map(String::from),
            send_email: params["send_email"].as_bool(),
        };

        let report = ReportService::generate_report(
            pool,
            crypto,
            gemini,
            email_service,
            user_id,
            user_name,
            user_email,
            encryption_salt,
            &req,
            "agentic",
        )
        .await?;

        Ok(serde_json::to_value(&report).unwrap_or(json!({"success": true})))
    }

    async fn tool_get_baseline(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
    ) -> std::result::Result<Value, AppError> {
        let baseline = crate::services::onboarding_service::OnboardingService::get_baseline(
            pool,
            crypto,
            user_id,
            encryption_salt,
        )
        .await?;

        Ok(serde_json::to_value(&baseline).unwrap_or(json!({"success": true})))
    }

    // ── Note CRUD Tools ─────────────────────────────────────

    async fn tool_create_note(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let title = params["title"]
            .as_str()
            .unwrap_or("Untitled Note")
            .to_string();
        let content = params["content"]
            .as_str()
            .ok_or_else(|| AppError::BadRequest("CREATE_NOTE requires 'content' parameter".into()))?
            .to_string();
        let label = params["label"].as_str().map(String::from);
        let is_pinned = params["is_pinned"].as_bool();

        let req = crate::models::note::CreateNoteRequest {
            title,
            content,
            label,
            is_pinned,
        };

        let note = NoteService::create_note(pool, crypto, user_id, encryption_salt, &req).await?;

        Ok(json!({
            "success": true,
            "note": note,
            "message": "Note created successfully"
        }))
    }

    async fn tool_update_note(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let note_id = params["note_id"].as_str().ok_or_else(|| {
            AppError::BadRequest("UPDATE_NOTE requires 'note_id' parameter".into())
        })?;

        let req = crate::models::note::UpdateNoteRequest {
            title: params["title"].as_str().map(String::from),
            content: params["content"].as_str().map(String::from),
            label: params["label"].as_str().map(String::from),
            is_pinned: params["is_pinned"].as_bool(),
        };

        let note =
            NoteService::update_note(pool, crypto, user_id, note_id, encryption_salt, &req).await?;

        Ok(json!({
            "success": true,
            "note": note,
            "message": "Note updated successfully"
        }))
    }

    async fn tool_delete_note(
        pool: &MySqlPool,
        user_id: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let note_id = params["note_id"].as_str().ok_or_else(|| {
            AppError::BadRequest("DELETE_NOTE requires 'note_id' parameter".into())
        })?;

        NoteService::delete_note(pool, user_id, note_id).await?;

        Ok(json!({
            "success": true,
            "message": "Note deleted successfully"
        }))
    }

    // ── Coping Strategy Suggestion Tool ──────────────────────

    async fn tool_suggest_coping_strategies(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        user_id: &str,
        user_name: &str,
        encryption_salt: &str,
        params: &Value,
    ) -> std::result::Result<Value, AppError> {
        let context = params["context"].as_str().unwrap_or("general wellness");
        let save_as_notes = params["save_as_notes"].as_bool().unwrap_or(false);
        let label = params["label"].as_str().unwrap_or("coping");

        let entries = crate::services::mood_service::MoodService::get_mood_entries(
            pool,
            crypto,
            user_id,
            encryption_salt,
            None,
            None,
            Some(14),
        )
        .await
        .unwrap_or_default();

        let existing_notes =
            NoteService::get_notes(pool, crypto, user_id, encryption_salt, Some("coping"), 20)
                .await
                .unwrap_or_default();

        let existing_titles: Vec<String> = existing_notes.iter().map(|n| n.title.clone()).collect();

        let baseline = crate::services::onboarding_service::OnboardingService::get_baseline(
            pool,
            crypto,
            user_id,
            encryption_salt,
        )
        .await
        .ok();

        let prompt = format!(
            r#"Generate personalized coping strategies for user "{name}".

CONTEXT FROM USER: {context}

RECENT MOOD DATA (last 14 days):
{mood_data}

BASELINE PROFILE:
{baseline}

EXISTING COPING NOTES (avoid duplicating these):
{existing}

Generate 3-5 specific, actionable coping strategies as JSON:
{{
  "strategies": [
    {{
      "title": "Short descriptive title (max 50 chars)",
      "content": "Detailed step-by-step instructions (2-3 paragraphs). Include when to use it, how to do it, and why it helps.",
      "category": "breathing|mindfulness|physical|journaling|social|creative|cognitive|routine"
    }}
  ]
}}

Tailor strategies to the user's specific situation, stress levels, and patterns.
Be practical, empathetic, and evidence-based. Return ONLY valid JSON."#,
            name = user_name,
            context = context,
            mood_data = serde_json::to_string_pretty(&entries).unwrap_or_default(),
            baseline = serde_json::to_string_pretty(&baseline).unwrap_or("Not available".into()),
            existing = existing_titles.join(", ")
        );

        let ai_raw = gemini
            .generate_with_system(
                &prompt,
                Some(
                    "You are a mental health coping strategy expert. Always respond with valid JSON only."
                ),
                0.6,
                4096
            ).await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let parsed = Self::parse_strategies_json(&ai_raw)?;
        let strategies = parsed["strategies"].as_array();

        let mut saved_notes = Vec::new();

        if save_as_notes {
            if let Some(strategies_arr) = strategies {
                for strategy in strategies_arr {
                    let title = strategy["title"].as_str().unwrap_or("Coping Strategy");
                    let content = strategy["content"].as_str().unwrap_or("");
                    let category = strategy["category"].as_str().unwrap_or(label);

                    if content.is_empty() {
                        continue;
                    }

                    let req = crate::models::note::CreateNoteRequest {
                        title: title.to_string(),
                        content: content.to_string(),
                        label: Some(category.to_string()),
                        is_pinned: Some(false),
                    };

                    match NoteService::create_note(pool, crypto, user_id, encryption_salt, &req)
                        .await
                    {
                        Ok(note) => saved_notes.push(json!({
                            "id": note.id,
                            "title": note.title,
                            "label": note.label,
                        })),
                        Err(e) => {
                            tracing::warn!("Failed to save coping note: {:?}", e);
                        }
                    }
                }
            }
        }

        Ok(json!({
            "success": true,
            "strategies": parsed["strategies"],
            "saved_as_notes": save_as_notes,
            "saved_notes": saved_notes,
            "count": strategies.map(|s| s.len()).unwrap_or(0)
        }))
    }

    fn parse_strategies_json(raw: &str) -> std::result::Result<Value, AppError> {
        let trimmed = raw.trim();
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            return Ok(v);
        }
        let cleaned = trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str::<Value>(cleaned).map_err(|e| {
            tracing::error!(
                "Failed to parse coping strategies JSON: {}. Raw: {}",
                e,
                raw
            );
            AppError::InternalError(anyhow::anyhow!("Failed to parse AI coping strategies"))
        })
    }

    // ── System Prompt ───────────────────────────────────────

    fn build_agent_system_prompt(user_name: &str) -> String {
        format!(
            r#"You are BSDY Agentic AI, an advanced mental health assistant with access to user data tools. You are helping {name}.

YOUR CAPABILITIES (Tools):

--- DATA RETRIEVAL ---
1. GET_MOOD_ENTRIES — Retrieve mood tracker data
   Parameters: from (YYYY-MM-DD), to (YYYY-MM-DD), limit (number)
2. GET_ANALYTICS — Get existing AI analytics summaries
   Parameters: limit (number)
3. GET_NOTES — Get user's coping toolkit notes
   Parameters: label (optional filter), limit (number)
4. GET_REPORTS — Get existing mental health reports
   Parameters: limit (number)
5. GET_BASELINE — Get user's baseline mental health assessment
   Parameters: (none)

--- AI GENERATION ---
6. GENERATE_ANALYTICS — Generate a new AI analytics summary
   Parameters: period (weekly|monthly|quarterly)
7. GENERATE_REPORT — Generate a new mental health report
   Parameters: report_type (weekly|monthly|yearly|custom), period_start (YYYY-MM-DD), period_end (YYYY-MM-DD), send_email (bool)
8. SUGGEST_COPING_STRATEGIES — Analyze user data and generate personalized coping strategies
   Parameters: context (what the user is struggling with), save_as_notes (bool — save strategies to their coping toolkit), label (note label, default "coping")

--- NOTE MANAGEMENT ---
9. CREATE_NOTE — Create a new coping toolkit note for the user
   Parameters: title (string), content (string), label (optional, e.g. "coping", "breathing", "journaling"), is_pinned (bool)
10. UPDATE_NOTE — Edit an existing note
    Parameters: note_id (required), title (optional), content (optional), label (optional), is_pinned (optional)
11. DELETE_NOTE — Delete a note
    Parameters: note_id (required)

RESPONSE FORMAT:
When you need to use tools, respond in this EXACT JSON format:
{{
  "response": "Brief explanation of what you're doing",
  "tool_calls": [
    {{
      "tool_name": "TOOL_NAME",
      "parameters": {{ "param": "value" }}
    }}
  ]
}}

When just responding conversationally without tools, respond in plain text.

GUIDELINES:
1. Use tools proactively when the user asks about their data, trends, or wants analysis
2. Combine multiple tool calls when needed to give comprehensive answers
3. After receiving tool results, summarize them in a friendly, natural way
4. Always be empathetic and constructive with mental health data
5. If mood data shows concerning patterns, address it with care and suggest professional help
6. When generating reports, ask the user which type (weekly, monthly, yearly) and confirm before sending emails
7. You can cross-reference different data sources (mood + notes + analytics) for deeper insights
8. If the user asks about something unrelated to their mental health data, respond conversationally
9. Never show raw JSON to the user — always present data naturally
10. Be proactive: if you notice something interesting in the data, mention it
11. When the user asks for coping strategies, use SUGGEST_COPING_STRATEGIES to generate personalized ones based on their mood data and baseline
12. Offer to save coping strategies as notes (set save_as_notes=true) so users can reference them later
13. When creating or editing notes, write in a warm, practical tone with clear actionable steps
14. When the user asks to create a note, use CREATE_NOTE with a descriptive title and structured content
15. When updating notes, first use GET_NOTES to find the note_id, then UPDATE_NOTE with the changes
16. For report requests: 'weekly' covers last 7 days, 'monthly' covers last 30 days, 'yearly' covers last 365 days"#,
            name = user_name
        )
    }
}
