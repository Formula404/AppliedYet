CREATE TABLE interview_experience_sources (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL CHECK(source_type IN ('link', 'manual')),
    url TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('待分析', '已提取', '分析失败')),
    questions_json TEXT NOT NULL DEFAULT '[]',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK((source_type = 'link' AND url IS NOT NULL) OR (source_type = 'manual' AND url IS NULL))
);

CREATE INDEX interview_experience_sources_application_idx
    ON interview_experience_sources(application_id, created_at DESC);
