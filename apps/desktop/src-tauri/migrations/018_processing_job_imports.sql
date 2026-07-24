CREATE TABLE IF NOT EXISTS processing_jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('document_parse', 'asr')),
    application_id TEXT REFERENCES applications(id),
    source_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'succeeded', 'failed')),
    result_json TEXT,
    error_message TEXT,
    duration_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT
);

ALTER TABLE processing_jobs ADD COLUMN import_status TEXT NOT NULL DEFAULT 'pending'
    CHECK(import_status IN ('pending', 'running', 'succeeded', 'failed', 'skipped'));
ALTER TABLE processing_jobs ADD COLUMN interview_session_id TEXT
    REFERENCES interview_sessions(id) ON DELETE SET NULL;
ALTER TABLE processing_jobs ADD COLUMN import_error_message TEXT;
ALTER TABLE processing_jobs ADD COLUMN import_started_at TEXT;
ALTER TABLE processing_jobs ADD COLUMN import_completed_at TEXT;

CREATE INDEX processing_jobs_created_idx ON processing_jobs(created_at DESC);
CREATE INDEX processing_jobs_application_idx
    ON processing_jobs(application_id, created_at DESC);
