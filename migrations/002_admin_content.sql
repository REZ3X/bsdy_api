-- BSDY Mental Companion & Tracker Platform
-- Migration 002: Admin role + Content management feature
-- Adds role attribute to users and creates content (blog) tables

SET NAMES utf8mb4;
SET CHARACTER SET utf8mb4;

-- ============================================================
-- ADD ROLE COLUMN TO USERS
-- ============================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS role ENUM('basic', 'admin') NOT NULL DEFAULT 'basic' AFTER onboarding_completed;

-- ============================================================
-- CONTENTS (Blog/Article Management - Admin Only)
-- ============================================================
CREATE TABLE IF NOT EXISTS contents (
    id CHAR(36) NOT NULL PRIMARY KEY,
    author_id CHAR(36) NOT NULL,
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL UNIQUE,
    body TEXT NOT NULL,
    excerpt VARCHAR(1000),
    cover_image VARCHAR(500),
    status ENUM('draft', 'published', 'archived') NOT NULL DEFAULT 'draft',
    published_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_contents_status (status, published_at DESC),
    INDEX idx_contents_slug (slug),
    INDEX idx_contents_author (author_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
