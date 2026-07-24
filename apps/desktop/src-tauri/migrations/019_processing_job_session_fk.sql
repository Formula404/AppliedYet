ALTER TABLE processing_jobs RENAME TO processing_jobs_before_session_fk_fix;

CREATE TABLE processing_jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('document_parse', 'asr')),
    application_id TEXT REFERENCES applications(id),
    source_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'succeeded', 'failed')),
    result_json TEXT,
    error_message TEXT,
    duration_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    import_status TEXT NOT NULL DEFAULT 'pending'
        CHECK(import_status IN ('pending', 'running', 'succeeded', 'failed', 'skipped')),
    interview_session_id TEXT REFERENCES interview_sessions(id) ON DELETE SET NULL,
    import_error_message TEXT,
    import_started_at TEXT,
    import_completed_at TEXT
);

INSERT INTO processing_jobs(
    id,kind,application_id,source_path,status,result_json,error_message,duration_ms,
    created_at,completed_at,import_status,interview_session_id,import_error_message,
    import_started_at,import_completed_at
)
SELECT
    id,kind,application_id,source_path,status,result_json,error_message,duration_ms,
    created_at,completed_at,import_status,interview_session_id,import_error_message,
    import_started_at,import_completed_at
FROM processing_jobs_before_session_fk_fix;

DROP TABLE processing_jobs_before_session_fk_fix;

CREATE INDEX processing_jobs_created_idx ON processing_jobs(created_at DESC);
CREATE INDEX processing_jobs_application_idx
    ON processing_jobs(application_id, created_at DESC);
