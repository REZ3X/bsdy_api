-- BSDY Mental Companion & Tracker Platform
-- Migration 003: Separate admin action logs from user activity logs
-- Admin actions (content management etc.) go to admin_action_logs
-- User activity logs remain for basic role actions only

SET NAMES utf8mb4;
SET CHARACTER SET utf8mb4;

-- ============================================================
-- ADMIN ACTION LOGS (Admin-only actions like content management)
-- ============================================================
CREATE TABLE IF NOT EXISTS admin_action_logs (
    id CHAR(36) NOT NULL PRIMARY KEY,
    admin_id CHAR(36) NOT NULL,
    action ENUM('create', 'read', 'update', 'delete') NOT NULL,
    feature VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id CHAR(36),
    details TEXT,
    ip_address VARCHAR(45),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (admin_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_admin_logs_admin (admin_id, created_at),
    INDEX idx_admin_logs_feature (feature, created_at),
    INDEX idx_admin_logs_entity (entity_type, entity_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
