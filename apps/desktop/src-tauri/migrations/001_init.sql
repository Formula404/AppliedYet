PRAGMA foreign_keys = ON;

CREATE TABLE companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT,
    industry TEXT,
    company_type TEXT,
    website TEXT,
    career_site TEXT,
    aliases_json TEXT NOT NULL DEFAULT '[]',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE UNIQUE INDEX companies_name_active_idx
    ON companies(name COLLATE NOCASE) WHERE deleted_at IS NULL;

CREATE TABLE resume_versions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE TABLE positions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    title TEXT NOT NULL,
    direction TEXT,
    department TEXT,
    location TEXT,
    recruitment_type TEXT,
    batch_name TEXT,
    job_code TEXT,
    source_url TEXT,
    jd_raw TEXT,
    responsibilities TEXT,
    requirements TEXT,
    bonus_points TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE INDEX positions_company_idx ON positions(company_id);

CREATE TABLE applications (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL REFERENCES positions(id),
    resume_version_id TEXT REFERENCES resume_versions(id),
    applied_at TEXT,
    channel TEXT,
    priority INTEGER NOT NULL DEFAULT 2 CHECK(priority BETWEEN 1 AND 3),
    current_stage TEXT NOT NULL DEFAULT '已投递',
    final_result TEXT,
    next_action TEXT,
    next_action_due_at TEXT,
    status_updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE INDEX applications_position_idx ON applications(position_id);
CREATE INDEX applications_stage_idx ON applications(current_stage) WHERE deleted_at IS NULL;

CREATE TABLE application_events (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id),
    event_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    source_type TEXT NOT NULL CHECK(source_type IN ('manual', 'email', 'ai', 'system')),
    source_id TEXT,
    stage_before TEXT,
    stage_after TEXT,
    happened_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    reversible INTEGER NOT NULL DEFAULT 0 CHECK(reversible IN (0, 1)),
    reverted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX application_events_application_idx
    ON application_events(application_id, happened_at DESC);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    application_id TEXT REFERENCES applications(id),
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 2 CHECK(priority BETWEEN 1 AND 3),
    status TEXT NOT NULL DEFAULT 'todo' CHECK(status IN ('todo', 'doing', 'done', 'canceled')),
    due_at TEXT,
    remind_at TEXT,
    source_type TEXT NOT NULL DEFAULT 'manual',
    source_id TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX tasks_due_idx ON tasks(status, due_at);
