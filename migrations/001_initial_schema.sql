-- BSDY Mental Companion & Tracker Platform
-- Initial Database Schema for MariaDB
-- Run: mysql -u root -p bsdy_db < migrations/001_initial_schema.sql

SET NAMES utf8mb4;
SET CHARACTER SET utf8mb4;

-- ============================================================
-- USERS
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
    id CHAR(36) NOT NULL PRIMARY KEY,
    google_id VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    avatar_url TEXT,
    birth DATE,
    email_verification_status ENUM('pending', 'verified') NOT NULL DEFAULT 'pending',
    email_verification_token VARCHAR(255),
    email_verified_at DATETIME,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    encryption_salt CHAR(32) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_users_email (email),
    INDEX idx_users_google (google_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- MENTAL CHARACTERISTICS (Baseline Assessment - Encrypted)
-- ============================================================
CREATE TABLE IF NOT EXISTS mental_characteristics (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    risk_level ENUM('low', 'moderate', 'high', 'severe') NOT NULL DEFAULT 'low',
    assessment_version INT NOT NULL DEFAULT 1,
    -- Encrypted fields (AES-256-GCM, base64 encoded)
    family_background_enc TEXT,
    stress_level_enc TEXT NOT NULL,
    anxiety_level_enc TEXT NOT NULL,
    depression_level_enc TEXT NOT NULL,
    sleep_quality_enc TEXT NOT NULL,
    social_support_enc TEXT NOT NULL,
    coping_style_enc TEXT NOT NULL,
    personality_traits_enc TEXT NOT NULL,
    mental_health_history_enc TEXT NOT NULL,
    current_medications_enc TEXT,
    therapy_status_enc TEXT NOT NULL,
    additional_notes_enc TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_mc_user (user_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- MOOD ENTRIES (Daily Mood Tracker)
-- ============================================================
CREATE TABLE IF NOT EXISTS mood_entries (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    entry_date DATE NOT NULL,
    mood_score TINYINT NOT NULL,
    energy_level TINYINT,
    anxiety_level TINYINT,
    stress_level TINYINT,
    sleep_hours DECIMAL(3,1),
    sleep_quality TINYINT,
    appetite ENUM('very_low', 'low', 'normal', 'high', 'very_high'),
    social_interaction BOOLEAN DEFAULT FALSE,
    exercise_done BOOLEAN DEFAULT FALSE,
    notes_enc TEXT,
    triggers_enc TEXT,
    activities_enc TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_mood_user_date (user_id, entry_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- MENTAL ANALYTICS SUMMARIES (AI-Generated)
-- ============================================================
CREATE TABLE IF NOT EXISTS mental_analytics_summaries (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    period_type ENUM('weekly', 'monthly', 'quarterly') NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    summary_enc TEXT NOT NULL,
    insights_enc TEXT NOT NULL,
    recommendations_enc TEXT NOT NULL,
    overall_mood_trend ENUM('improving', 'stable', 'declining') NOT NULL DEFAULT 'stable',
    avg_mood_score DECIMAL(3,1),
    risk_level ENUM('low', 'moderate', 'high', 'severe') NOT NULL DEFAULT 'low',
    generated_by ENUM('automatic', 'manual') NOT NULL DEFAULT 'automatic',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_analytics_user_period (user_id, period_type, period_start)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- MENTAL REPORTS
-- ============================================================
CREATE TABLE IF NOT EXISTS mental_reports (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    report_type ENUM('weekly', 'monthly', 'quarterly', 'custom') NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    title VARCHAR(255) NOT NULL DEFAULT 'Mental Health Report',
    content_enc TEXT NOT NULL,
    ai_analysis_enc TEXT NOT NULL,
    recommendations_enc TEXT NOT NULL,
    status ENUM('generated', 'sent', 'failed') NOT NULL DEFAULT 'generated',
    sent_via_email BOOLEAN NOT NULL DEFAULT FALSE,
    sent_at DATETIME,
    trigger_type ENUM('automatic', 'manual', 'agentic') NOT NULL DEFAULT 'automatic',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_reports_user (user_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- CHATS
-- ============================================================
CREATE TABLE IF NOT EXISTS chats (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    title VARCHAR(255) NOT NULL DEFAULT 'New Chat',
    chat_type ENUM('companion', 'agentic') NOT NULL DEFAULT 'companion',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    message_count INT NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_chats_user (user_id, updated_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- CHAT MESSAGES (Encrypted)
-- ============================================================
CREATE TABLE IF NOT EXISTS chat_messages (
    id CHAR(36) NOT NULL PRIMARY KEY,
    chat_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    role ENUM('user', 'assistant', 'system') NOT NULL,
    content_enc TEXT NOT NULL,
    tool_calls_enc TEXT,
    tool_results_enc TEXT,
    has_tool_calls BOOLEAN NOT NULL DEFAULT FALSE,
    severity_flag ENUM('none', 'mild', 'moderate', 'severe', 'crisis') NOT NULL DEFAULT 'none',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_messages_chat (chat_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- NOTES (Coping Toolkit - Encrypted)
-- ============================================================
CREATE TABLE IF NOT EXISTS notes (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    title_enc TEXT NOT NULL,
    content_enc TEXT NOT NULL,
    label VARCHAR(100),
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_notes_user (user_id, updated_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- USER AUTH LOGS
-- ============================================================
CREATE TABLE IF NOT EXISTS user_auth_logs (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    action ENUM('login', 'logout', 'token_refresh', 'email_verify', 'verification_sent') NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    failure_reason VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_auth_logs_user (user_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- USER ACTIVITY LOGS
-- ============================================================
CREATE TABLE IF NOT EXISTS user_activity_logs (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    action ENUM('create', 'read', 'update', 'delete') NOT NULL,
    feature VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id CHAR(36),
    details TEXT,
    ip_address VARCHAR(45),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_activity_user (user_id, created_at),
    INDEX idx_activity_entity (entity_type, entity_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- SCHEDULED TASKS TRACKING
-- ============================================================
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id CHAR(36) NOT NULL PRIMARY KEY,
    task_type VARCHAR(100) NOT NULL,
    last_run_at DATETIME,
    next_run_at DATETIME NOT NULL,
    status ENUM('pending', 'running', 'completed', 'failed') NOT NULL DEFAULT 'pending',
    details TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
