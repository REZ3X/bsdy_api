use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::chat::{
        ChatMessageResponse,
        ChatMessageRow,
        ChatResponse,
        ChatRow,
        CreateChatRequest,
        SendMessageRequest,
        UpdateChatRequest,
    },
    services::gemini_service::GeminiService,
};

/// Severity keywords used for crisis detection in chat messages.
const CRISIS_KEYWORDS: &[&str] = &[
    "suicide",
    "kill myself",
    "end my life",
    "want to die",
    "don't want to live",
    "self-harm",
    "hurt myself",
    "cutting myself",
    "no reason to live",
    "better off dead",
    "can't go on",
    "end it all",
];

const SEVERE_KEYWORDS: &[&str] = &[
    "hopeless",
    "worthless",
    "can't take it",
    "giving up",
    "nothing matters",
    "all alone",
    "nobody cares",
    "unbearable",
    "breaking down",
];

pub struct ChatService;

impl ChatService {
    // ── Chat CRUD ───────────────────────────────────────────

    /// Create a new chat session.
    pub async fn create_chat(
        pool: &MySqlPool,
        user_id: &str,
        req: &CreateChatRequest
    ) -> Result<ChatResponse> {
        let id = Uuid::new_v4().to_string();
        let chat_type = req.chat_type.as_deref().unwrap_or("companion");

        if !["companion", "agentic"].contains(&chat_type) {
            return Err(
                AppError::ValidationError("chat_type must be 'companion' or 'agentic'".into())
            );
        }

        sqlx
            ::query(
                r#"INSERT INTO chats (id, user_id, title, chat_type, is_active, message_count)
               VALUES (?, ?, 'New Chat', ?, TRUE, 0)"#
            )
            .bind(&id)
            .bind(user_id)
            .bind(chat_type)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ?")
            .bind(&id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Ok(ChatResponse::from(&row))
    }

    /// List all chats for a user.
    pub async fn list_chats(
        pool: &MySqlPool,
        user_id: &str,
        limit: i64
    ) -> Result<Vec<ChatResponse>> {
        let rows: Vec<ChatRow> = sqlx
            ::query_as(
                r#"SELECT * FROM chats
               WHERE user_id = ?
               ORDER BY updated_at DESC
               LIMIT ?"#
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        Ok(rows.iter().map(ChatResponse::from).collect())
    }

    /// Get a single chat.
    pub async fn get_chat(pool: &MySqlPool, user_id: &str, chat_id: &str) -> Result<ChatResponse> {
        let row: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ? AND user_id = ?")
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Chat not found".into()))?;

        Ok(ChatResponse::from(&row))
    }

    /// Update a chat title or active status.
    pub async fn update_chat(
        pool: &MySqlPool,
        user_id: &str,
        chat_id: &str,
        req: &UpdateChatRequest
    ) -> Result<ChatResponse> {
        // Verify ownership
        let _: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ? AND user_id = ?")
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Chat not found".into()))?;

        sqlx
            ::query(
                r#"UPDATE chats SET
                title = COALESCE(?, title),
                is_active = COALESCE(?, is_active),
                updated_at = NOW()
               WHERE id = ? AND user_id = ?"#
            )
            .bind(req.title.as_deref())
            .bind(req.is_active)
            .bind(chat_id)
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Ok(ChatResponse::from(&row))
    }

    /// Delete a chat and all its messages.
    pub async fn delete_chat(pool: &MySqlPool, user_id: &str, chat_id: &str) -> Result<()> {
        let result = sqlx
            ::query("DELETE FROM chats WHERE id = ? AND user_id = ?")
            .bind(chat_id)
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Chat not found".into()));
        }
        Ok(())
    }

    // ── Messages ────────────────────────────────────────────

    /// Get messages for a chat in chronological order.
    pub async fn get_messages(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        chat_id: &str,
        encryption_salt: &str,
        limit: i64
    ) -> Result<Vec<ChatMessageResponse>> {
        // Verify ownership
        let _: ChatRow = sqlx
            ::query_as("SELECT * FROM chats WHERE id = ? AND user_id = ?")
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Chat not found".into()))?;

        let rows: Vec<ChatMessageRow> = sqlx
            ::query_as(
                r#"SELECT * FROM chat_messages
               WHERE chat_id = ? AND user_id = ?
               ORDER BY created_at ASC
               LIMIT ?"#
            )
            .bind(chat_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        rows.iter()
            .map(|r| Self::decrypt_message(crypto, r, encryption_salt))
            .collect()
    }

    /// Send a message in a companion chat and get AI response.
    pub async fn send_companion_message(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        user_id: &str,
        user_name: &str,
        chat_id: &str,
        encryption_salt: &str,
        req: &SendMessageRequest
    ) -> Result<(ChatMessageResponse, ChatMessageResponse)> {
        // Verify ownership and get chat
        let chat: ChatRow = sqlx
            ::query_as(
                "SELECT * FROM chats WHERE id = ? AND user_id = ? AND chat_type = 'companion'"
            )
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Companion chat not found".into()))?;

        // Detect severity
        let severity = Self::detect_severity(&req.message);

        // Save user message
        let user_msg = Self::save_message(
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

        // Load recent conversation history for context (last 20 messages)
        let history_rows: Vec<ChatMessageRow> = sqlx
            ::query_as(
                r#"SELECT * FROM chat_messages
               WHERE chat_id = ? AND user_id = ?
               ORDER BY created_at DESC
               LIMIT 20"#
            )
            .bind(chat_id)
            .bind(user_id)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        // Decrypt and reverse to chronological order
        let mut history: Vec<(String, String)> = history_rows
            .iter()
            .rev()
            .filter_map(|row| {
                let content = crypto.decrypt(&row.content_enc, encryption_salt).ok()?;
                let role = if row.role == "user" {
                    "user".to_string()
                } else {
                    "model".to_string()
                };
                Some((role, content))
            })
            .collect();

        // Remove the last entry since it's the current user message we just saved
        // (it's already in the history from the DB but we'll send it as the current message)
        if !history.is_empty() && history.last().map(|(r, _)| r.as_str()) == Some("user") {
            history.pop();
        }

        // Build system prompt
        let system_prompt = Self::build_companion_system_prompt(user_name, &severity);

        // Call Gemini
        let ai_response = gemini
            .generate_chat_response(&system_prompt, &history, &req.message, 0.8).await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Save assistant message
        let assistant_msg = Self::save_message(
            pool,
            crypto,
            chat_id,
            user_id,
            "assistant",
            &ai_response,
            None,
            None,
            "none",
            encryption_salt
        ).await?;

        // Update chat message count and title if first message
        let new_count = chat.message_count + 2;
        sqlx::query("UPDATE chats SET message_count = ?, updated_at = NOW() WHERE id = ?")
            .bind(new_count)
            .bind(chat_id)
            .execute(pool).await
            .ok();

        // Auto-generate title on first message
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

    // ── Helpers ─────────────────────────────────────────────

    pub async fn save_message(
        pool: &MySqlPool,
        crypto: &CryptoService,
        chat_id: &str,
        user_id: &str,
        role: &str,
        content: &str,
        tool_calls: Option<&serde_json::Value>,
        tool_results: Option<&serde_json::Value>,
        severity: &str,
        encryption_salt: &str
    ) -> Result<ChatMessageResponse> {
        let id = Uuid::new_v4().to_string();
        let content_enc = crypto.encrypt(content, encryption_salt)?;
        let tool_calls_enc = tool_calls
            .map(|tc| {
                let json = serde_json::to_string(tc).unwrap_or_default();
                crypto.encrypt(&json, encryption_salt)
            })
            .transpose()?;
        let tool_results_enc = tool_results
            .map(|tr| {
                let json = serde_json::to_string(tr).unwrap_or_default();
                crypto.encrypt(&json, encryption_salt)
            })
            .transpose()?;
        let has_tool_calls = tool_calls.is_some();

        sqlx
            ::query(
                r#"INSERT INTO chat_messages
               (id, chat_id, user_id, role, content_enc, tool_calls_enc, tool_results_enc,
                has_tool_calls, severity_flag)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
            )
            .bind(&id)
            .bind(chat_id)
            .bind(user_id)
            .bind(role)
            .bind(&content_enc)
            .bind(tool_calls_enc.as_deref())
            .bind(tool_results_enc.as_deref())
            .bind(has_tool_calls)
            .bind(severity)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        Ok(ChatMessageResponse {
            id,
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            tool_calls: tool_calls.cloned(),
            tool_results: tool_results.cloned(),
            has_tool_calls,
            severity_flag: severity.to_string(),
            created_at: chrono::Local::now().naive_local().to_string(),
        })
    }

    fn decrypt_message(
        crypto: &CryptoService,
        row: &ChatMessageRow,
        salt: &str
    ) -> Result<ChatMessageResponse> {
        let content = crypto.decrypt(&row.content_enc, salt)?;
        let tool_calls = if let Some(ref enc) = row.tool_calls_enc {
            let json_str = crypto.decrypt(enc, salt)?;
            serde_json::from_str(&json_str).ok()
        } else {
            None
        };
        let tool_results = if let Some(ref enc) = row.tool_results_enc {
            let json_str = crypto.decrypt(enc, salt)?;
            serde_json::from_str(&json_str).ok()
        } else {
            None
        };

        Ok(ChatMessageResponse {
            id: row.id.clone(),
            chat_id: row.chat_id.clone(),
            role: row.role.clone(),
            content,
            tool_calls,
            tool_results,
            has_tool_calls: row.has_tool_calls,
            severity_flag: row.severity_flag.clone(),
            created_at: row.created_at.to_string(),
        })
    }

    /// Detect message severity for crisis threshold.
    pub fn detect_severity(message: &str) -> String {
        let lower = message.to_lowercase();

        for keyword in CRISIS_KEYWORDS {
            if lower.contains(keyword) {
                return "crisis".to_string();
            }
        }
        for keyword in SEVERE_KEYWORDS {
            if lower.contains(keyword) {
                return "severe".to_string();
            }
        }
        "none".to_string()
    }

    fn build_companion_system_prompt(user_name: &str, severity: &str) -> String {
        let crisis_instruction = match severity {
            "crisis" | "severe" =>
                r#"

[CRITICAL] The user's message indicates they may be in crisis or severe distress.
Your IMMEDIATE priorities are:
1. Acknowledge their pain with deep empathy
2. Ask if they are safe right now
3. Provide crisis hotline numbers:
   - Into The Light Indonesia: 119 ext 8
   - International: https://www.iasp.info/resources/Crisis_Centres/
4. Strongly encourage them to reach out to a qualified professional
5. DO NOT try to be their therapist — you are a companion, not a replacement for professional help
6. Stay present, warm, and non-judgmental

"#,
            _ => "",
        };

        format!(
            r#"You are BSDY, a compassionate mental health companion & caregiver AI. You are talking to {name}.

YOUR ROLE:
- You are a warm, empathetic, and supportive mental wellness companion
- You listen actively, validate feelings, and offer gentle guidance
- You are NOT a therapist or psychiatrist — you cannot diagnose or prescribe
- You help users process their thoughts, reflect on their emotions, and develop healthy coping strategies

GUIDELINES:
1. Always be empathetic, non-judgmental, and kind
2. Use the user's name naturally in conversation
3. Ask thoughtful follow-up questions to understand deeper
4. Validate emotions before offering suggestions
5. Suggest healthy coping strategies when appropriate (breathing exercises, journaling, movement)
6. Encourage professional help when patterns suggest it would be beneficial
7. Keep responses warm but concise (2-4 paragraphs max)
8. Keep a warm and gentle tone but not excessively
9. If the user shares something positive, celebrate it with them
10. Remember context from the conversation history
{crisis}
BOUNDARIES:
- Never diagnose mental health conditions
- Never recommend specific medications
- If someone is at risk, ALWAYS provide crisis resources first
- Don't promise things will "get better" — instead validate and offer small practical steps
- If asked about topics outside mental wellness, gently redirect

CONVERSATION STYLE:
- Warm and natural, like a caring friend
- Use active listening techniques (reflect back what they said)
- Balance between listening and offering gentle guidance
- Ask one question at a time to avoid overwhelming"#,
            name = user_name,
            crisis = crisis_instruction
        )
    }
}
