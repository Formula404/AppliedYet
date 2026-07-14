CREATE TABLE resume_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT,
    file_format TEXT,
    parsed_text TEXT NOT NULL DEFAULT '',
    personal_info TEXT NOT NULL DEFAULT '',
    education_background TEXT NOT NULL DEFAULT '',
    internship_experience TEXT NOT NULL DEFAULT '',
    project_experience TEXT NOT NULL DEFAULT '',
    professional_skills TEXT NOT NULL DEFAULT '',
    academic_achievements TEXT NOT NULL DEFAULT '',
    skill_certificates TEXT NOT NULL DEFAULT '',
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK(is_primary IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE INDEX resume_profiles_updated_idx ON resume_profiles(updated_at DESC) WHERE deleted_at IS NULL;
