use serde_json::{ json, Value };
use sqlx::MySqlPool;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::chat::{
        AgentResponse,
        ChatMessageResponse,
        ChatMessageRow,
        ChatRow,
        SendMessageRequest,
        ToolCall,
        ToolCallRequest,
        ToolResult,
    },
    services::{
        analytics_service::AnalyticsService,
        chat_service::ChatService,
        gemini_service::GeminiService,
        note_service::NoteService,
        report_service::ReportService,
        email_service::EmailService,
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
        req: &SendMessageRequest
    ) -> Result<(ChatMessageResponse, ChatMessageResponse)> {
        // Verify ownership and chat type
        let chat: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ? AND user_id = ? AND chat_type = 'agentic'")
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Agentic chat not found".into()))?;

        // Save user message
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
            encryption_salt
        ).await?;

        // Load conversation history
        let history_rows: Vec<ChatMessageRow> = sqlx
            ::query_as(
                r#"SELECT * FROM chat_messages
               WHERE chat_id = ? AND user_id = ?
               ORDER BY created_at DESC LIMIT 16"#
            )
            .bind(chat_id)
            .bind(user_id)
            .fetch_all(pool).await
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

        // Pop the last user msg since we'll pass it as current message
        if history.last().map(|(r, _)| r.as_str()) == Some("user") {
            history.pop();
        }

        let system_prompt = Self::build_agent_system_prompt(user_name);

        // First call — AI decides whether to use tools
        let ai_response = gemini
            .generate_chat_response(&system_prompt, &history, &req.message, 0.5).await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Parse tool calls
        let (response_text, tool_calls, tool_results) = Self::parse_and_execute_tools(
            pool,
            crypto,
            gemini,
            email_service,
            user_id,
            user_name,
            user_email,
            encryption_salt,
            &ai_response
        ).await?;

        // If there were tool calls, do a follow-up call for a natural response
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

        // Save assistant message with tool call data
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
            encryption_salt
        ).await?;

        // Update chat
        let new_count = chat.message_count + 2;
        sqlx::query("UPDATE chats SET message_count = ?, updated_at = NOW() WHERE id = ?")
            .bind(new_count)
            .bind(chat_id)
            .execute(pool).await
            .ok();

        if chat.message_count == 0 {
            if let Ok(title) = gemini.generate_chat_title(&req.message).await {
                sqlx::query("UPDATE chats SET title = ? WHERE id = ?")
                    .bind(&title)
                    .bind(chat_id)
                    .execute(pool).await
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
        ai_response: &str
    ) -> Result<(String, Vec<ToolCall>, Vec<ToolResult>)> {
        // Try to parse as JSON with tool calls
        let parsed = serde_json::from_str::<AgentResponse>(ai_response.trim());

        // Also try extracting JSON from markdown code block
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
                        tc
                    ).await;

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

        // Plain text response — no tool calls
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
        tool_call: &ToolCallRequest
    ) -> std::result::Result<Value, AppError> {
        match tool_call.tool_name.to_uppercase().as_str() {
            "GET_MOOD_ENTRIES" | "GET_MOOD_LOGS" => {
                Self::tool_get_mood_entries(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters
                ).await
            }
            "GET_ANALYTICS" | "GET_ANALYTICS_SUMMARY" => {
                Self::tool_get_analytics(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters
                ).await
            }
            "GENERATE_ANALYTICS" => {
                Self::tool_generate_analytics(
                    pool,
                    crypto,
                    gemini,
                    user_id,
                    user_name,
                    encryption_salt,
                    &tool_call.parameters
                ).await
            }
            "GET_NOTES" | "GET_COPING_NOTES" => {
                Self::tool_get_notes(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters
                ).await
            }
            "GET_REPORTS" => {
                Self::tool_get_reports(
                    pool,
                    crypto,
                    user_id,
                    encryption_salt,
                    &tool_call.parameters
                ).await
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
                    &tool_call.parameters
                ).await
            }
            "GET_BASELINE" | "GET_MENTAL_PROFILE" => {
                Self::tool_get_baseline(pool, crypto, user_id, encryption_salt).await
            }
            _ => Err(AppError::BadRequest(format!("Unknown tool: {}", tool_call.tool_name))),
        }
    }

    // ── Tool Implementations ────────────────────────────────

    async fn tool_get_mood_entries(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value
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
            Some(limit)
        ).await?;

        Ok(
            json!({
            "success": true,
            "entries": entries,
            "count": entries.len()
        })
        )
    }

    async fn tool_get_analytics(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value
    ) -> std::result::Result<Value, AppError> {
        let limit = params["limit"].as_i64().unwrap_or(5);

        let summaries = AnalyticsService::get_summaries(
            pool,
            crypto,
            user_id,
            encryption_salt,
            limit
        ).await?;

        Ok(
            json!({
            "success": true,
            "summaries": summaries,
            "count": summaries.len()
        })
        )
    }

    async fn tool_generate_analytics(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        user_id: &str,
        user_name: &str,
        encryption_salt: &str,
        params: &Value
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
            "agentic"
        ).await?;

        Ok(serde_json::to_value(&summary).unwrap_or(json!({"success": true})))
    }

    async fn tool_get_notes(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value
    ) -> std::result::Result<Value, AppError> {
        let label = params["label"].as_str();
        let limit = params["limit"].as_i64().unwrap_or(20);

        let notes = NoteService::get_notes(
            pool,
            crypto,
            user_id,
            encryption_salt,
            label,
            limit
        ).await?;

        Ok(
            json!({
            "success": true,
            "notes": notes,
            "count": notes.len()
        })
        )
    }

    async fn tool_get_reports(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        params: &Value
    ) -> std::result::Result<Value, AppError> {
        let limit = params["limit"].as_i64().unwrap_or(5);

        let reports = ReportService::get_reports(
            pool,
            crypto,
            user_id,
            encryption_salt,
            limit
        ).await?;

        Ok(
            json!({
            "success": true,
            "reports": reports,
            "count": reports.len()
        })
        )
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
        params: &Value
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
            "agentic"
        ).await?;

        Ok(serde_json::to_value(&report).unwrap_or(json!({"success": true})))
    }

    async fn tool_get_baseline(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str
    ) -> std::result::Result<Value, AppError> {
        let baseline = crate::services::onboarding_service::OnboardingService::get_baseline(
            pool,
            crypto,
            user_id,
            encryption_salt
        ).await?;

        Ok(serde_json::to_value(&baseline).unwrap_or(json!({"success": true})))
    }

    // ── System Prompt ───────────────────────────────────────

    fn build_agent_system_prompt(user_name: &str) -> String {
        format!(
            r#"You are BSDY Agentic AI, an advanced mental health assistant with access to user data tools. You are helping {name}.

YOUR CAPABILITIES (Tools):
1. GET_MOOD_ENTRIES — Retrieve mood tracker data
   Parameters: from (YYYY-MM-DD), to (YYYY-MM-DD), limit (number)
2. GET_ANALYTICS — Get existing AI analytics summaries
   Parameters: limit (number)
3. GENERATE_ANALYTICS — Generate a new AI analytics summary
   Parameters: period (weekly|monthly|quarterly)
4. GET_NOTES — Get user's coping toolkit notes
   Parameters: label (optional filter), limit (number)
5. GET_REPORTS — Get existing mental health reports
   Parameters: limit (number)
6. GENERATE_REPORT — Generate a new mental health report
   Parameters: report_type (weekly|monthly|quarterly|custom), period_start, period_end, send_email (bool)
7. GET_BASELINE — Get user's baseline mental health assessment

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
6. When generating reports, ask before sending emails
7. You can cross-reference different data sources (mood + notes + analytics) for deeper insights
8. If the user asks about something unrelated to their mental health data, respond conversationally
9. Never show raw JSON to the user — always present data naturally
10. Be proactive: if you notice something interesting in the data, mention it"#,
            name = user_name
        )
    }
}
