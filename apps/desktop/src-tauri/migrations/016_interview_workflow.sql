CREATE TABLE interview_sessions (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    session_type TEXT NOT NULL CHECK(session_type IN ('模拟面试', '真实面试')),
    round TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('进行中', '待复盘', '复盘完成')),
    current_question_index INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    review_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX interview_sessions_application_idx
    ON interview_sessions(application_id, created_at DESC);

CREATE TABLE interview_session_questions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES interview_sessions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    prompt TEXT NOT NULL,
    source TEXT NOT NULL,
    answer TEXT NOT NULL DEFAULT '',
    score INTEGER CHECK(score BETWEEN 0 AND 100),
    evaluation TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(session_id, position)
);

CREATE TABLE question_bank_items (
    id TEXT PRIMARY KEY,
    normalized_key TEXT NOT NULL UNIQUE,
    prompt TEXT NOT NULL,
    category TEXT NOT NULL,
    best_answer TEXT NOT NULL DEFAULT '',
    mastery TEXT NOT NULL DEFAULT '待加强' CHECK(mastery IN ('待加强', '练习中', '熟悉', '掌握')),
    source TEXT NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX question_bank_items_mastery_idx
    ON question_bank_items(mastery, last_seen_at DESC);
