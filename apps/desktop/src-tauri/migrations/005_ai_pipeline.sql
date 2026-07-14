CREATE TABLE ai_calls (
    id TEXT PRIMARY KEY,
    application_id TEXT REFERENCES applications(id),
    feature TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'succeeded', 'failed')),
    attempts INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER,
    input_sources_json TEXT NOT NULL DEFAULT '[]',
    response_json TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT
);

CREATE INDEX ai_calls_application_idx ON ai_calls(application_id, created_at DESC);

CREATE TABLE interview_preparations (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id),
    ai_call_id TEXT NOT NULL REFERENCES ai_calls(id),
    content_json TEXT NOT NULL,
    source_snapshot_json TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX interview_preparations_application_idx
    ON interview_preparations(application_id, created_at DESC);

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
    completed_at TEXT
);
