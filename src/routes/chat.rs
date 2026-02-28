use axum::{ extract::{ Path, Query, State }, routing::{ delete, get, post, put }, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::{ AppError, Result },
    middleware::{ activity_log::log_activity, auth::FullUser },
    models::chat::{ CreateChatRequest, SendMessageRequest, UpdateChatRequest },
    services::{ AgentService, ChatService },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_chat))
        .route("/", get(list_chats))
        .route("/{chat_id}", get(get_chat))
        .route("/{chat_id}", put(update_chat))
        .route("/{chat_id}", delete(delete_chat))
        .route("/{chat_id}/messages", get(get_messages))
        .route("/{chat_id}/messages", post(send_message))
}

#[derive(Debug, Deserialize)]
struct ChatListParams {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MessageListParams {
    limit: Option<i64>,
}

// ── POST /api/chats ─────────────────────────────────────────

async fn create_chat(
    State(state): State<AppState>,
    auth: FullUser,
    Json(req): Json<CreateChatRequest>
) -> Result<Json<Value>> {
    let chat = ChatService::create_chat(&state.db, &auth.user.id, &req).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "create",
        "chat",
        "chat",
        Some(&chat.id),
        Some(&format!("type: {}", chat.chat_type)),
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": chat })))
}

// ── GET /api/chats?limit= ──────────────────────────────────

async fn list_chats(
    State(state): State<AppState>,
    auth: FullUser,
    Query(params): Query<ChatListParams>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(20);
    let chats = ChatService::list_chats(&state.db, &auth.user.id, limit).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": chats,
        "count": chats.len()
    })
        )
    )
}

// ── GET /api/chats/:chat_id ────────────────────────────────

async fn get_chat(
    State(state): State<AppState>,
    auth: FullUser,
    Path(chat_id): Path<String>
) -> Result<Json<Value>> {
    let chat = ChatService::get_chat(&state.db, &auth.user.id, &chat_id).await?;
    Ok(Json(json!({ "success": true, "data": chat })))
}

// ── PUT /api/chats/:chat_id ────────────────────────────────

async fn update_chat(
    State(state): State<AppState>,
    auth: FullUser,
    Path(chat_id): Path<String>,
    Json(req): Json<UpdateChatRequest>
) -> Result<Json<Value>> {
    let chat = ChatService::update_chat(&state.db, &auth.user.id, &chat_id, &req).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "update",
        "chat",
        "chat",
        Some(&chat_id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": chat })))
}

// ── DELETE /api/chats/:chat_id ─────────────────────────────

async fn delete_chat(
    State(state): State<AppState>,
    auth: FullUser,
    Path(chat_id): Path<String>
) -> Result<Json<Value>> {
    ChatService::delete_chat(&state.db, &auth.user.id, &chat_id).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "delete",
        "chat",
        "chat",
        Some(&chat_id),
        None,
        None
    ).await;

    Ok(Json(json!({
        "success": true,
        "message": "Chat deleted"
    })))
}

// ── GET /api/chats/:chat_id/messages?limit= ────────────────

async fn get_messages(
    State(state): State<AppState>,
    auth: FullUser,
    Path(chat_id): Path<String>,
    Query(params): Query<MessageListParams>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(50);

    let messages = ChatService::get_messages(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &chat_id,
        &auth.user.encryption_salt,
        limit
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": messages,
        "count": messages.len()
    })
        )
    )
}

// ── POST /api/chats/:chat_id/messages ───────────────────────

async fn send_message(
    State(state): State<AppState>,
    auth: FullUser,
    Path(chat_id): Path<String>,
    Json(req): Json<SendMessageRequest>
) -> Result<Json<Value>> {
    if req.message.trim().is_empty() {
        return Err(AppError::ValidationError("Message cannot be empty".into()));
    }

    // Determine chat type to route to the correct service
    let chat = ChatService::get_chat(&state.db, &auth.user.id, &chat_id).await?;

    let (user_msg, assistant_msg) = match chat.chat_type.as_str() {
        "agentic" => {
            AgentService::process_message(
                &state.db,
                &state.crypto,
                &state.gemini,
                &state.email,
                &auth.user.id,
                &auth.user.name,
                &auth.user.email,
                &chat_id,
                &auth.user.encryption_salt,
                &req
            ).await?
        }
        _ => {
            // Default: companion chat
            ChatService::send_companion_message(
                &state.db,
                &state.crypto,
                &state.gemini,
                &auth.user.id,
                &auth.user.name,
                &chat_id,
                &auth.user.encryption_salt,
                &req
            ).await?
        }
    };

    log_activity(
        &state.db,
        &auth.user.id,
        "create",
        "chat",
        "chat_message",
        Some(&chat_id),
        Some(&format!("type: {}", chat.chat_type)),
        None
    ).await;

    Ok(
        Json(
            json!({
        "success": true,
        "data": {
            "user_message": user_msg,
            "assistant_message": assistant_msg
        }
    })
        )
    )
}
