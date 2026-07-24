CREATE TABLE question_topics (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE canonical_questions (
    id TEXT PRIMARY KEY,
    topic_id TEXT REFERENCES question_topics(id) ON DELETE SET NULL,
    display_prompt TEXT NOT NULL,
    question_type TEXT NOT NULL,
    system_mastery TEXT NOT NULL DEFAULT '待加强'
        CHECK(system_mastery IN ('待加强','练习中','熟悉','掌握')),
    manual_mastery TEXT CHECK(manual_mastery IN ('待加强','练习中','熟悉','掌握')),
    best_answer TEXT NOT NULL DEFAULT '',
    next_review_at TEXT,
    redirect_to_id TEXT REFERENCES canonical_questions(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE question_variants (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES canonical_questions(id) ON DELETE CASCADE,
    raw_prompt TEXT NOT NULL,
    normalized_prompt TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'und',
    confirmed_equivalent INTEGER NOT NULL DEFAULT 0 CHECK(confirmed_equivalent IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_seen_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE(question_id, normalized_prompt)
);
CREATE INDEX question_variants_normalized_idx ON question_variants(normalized_prompt);

CREATE TABLE question_observations (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES canonical_questions(id) ON DELETE CASCADE,
    variant_id TEXT REFERENCES question_variants(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL CHECK(event_type IN
        ('real_asked','mock_answered','reference_mentioned','manual_saved','ai_generated','imported_legacy')),
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_item_id TEXT NOT NULL,
    application_id TEXT REFERENCES applications(id) ON DELETE SET NULL,
    company_id TEXT,
    position_id TEXT,
    round TEXT,
    occurred_at TEXT NOT NULL,
    verification_state TEXT NOT NULL DEFAULT 'confirmed'
        CHECK(verification_state IN ('confirmed','inferred','pending','rejected')),
    confidence REAL,
    legacy_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE(event_type, source_type, source_id, source_item_id)
);
CREATE INDEX question_observations_question_idx
    ON question_observations(question_id,event_type,verification_state,occurred_at DESC);
CREATE INDEX question_observations_source_idx
    ON question_observations(source_type,source_id);

CREATE TABLE question_bank_memberships (
    question_id TEXT PRIMARY KEY REFERENCES canonical_questions(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','archived')),
    added_reason TEXT NOT NULL CHECK(added_reason IN
        ('manual','real_interview','mock_practice','reference_confirmed','legacy')),
    added_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    archived_at TEXT
);
CREATE INDEX question_bank_memberships_status_idx ON question_bank_memberships(status,added_at DESC);

CREATE TABLE question_match_decisions (
    id TEXT PRIMARY KEY,
    left_question_id TEXT NOT NULL REFERENCES canonical_questions(id) ON DELETE CASCADE,
    right_question_id TEXT NOT NULL REFERENCES canonical_questions(id) ON DELETE CASCADE,
    system_decision TEXT NOT NULL,
    confidence REAL,
    reason TEXT NOT NULL DEFAULT '',
    user_action TEXT,
    matcher_version TEXT NOT NULL DEFAULT 'local-v1',
    allow_resuggest INTEGER NOT NULL DEFAULT 1 CHECK(allow_resuggest IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE(left_question_id,right_question_id)
);

CREATE TABLE question_merge_audits (
    id TEXT PRIMARY KEY,
    target_question_id TEXT NOT NULL REFERENCES canonical_questions(id),
    source_question_id TEXT NOT NULL REFERENCES canonical_questions(id),
    snapshot_json TEXT NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    undone_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE question_answer_versions (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES canonical_questions(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    source_attempt_id TEXT,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK(is_current IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

ALTER TABLE interview_session_questions ADD COLUMN canonical_question_id TEXT
    REFERENCES canonical_questions(id) ON DELETE SET NULL;

INSERT INTO canonical_questions(
    id,display_prompt,question_type,system_mastery,manual_mastery,best_answer,created_at,updated_at
)
SELECT id,prompt,category,mastery,
       CASE WHEN source='手动' THEN mastery ELSE NULL END,
       best_answer,created_at,updated_at
FROM question_bank_items;

INSERT INTO question_variants(
    id,question_id,raw_prompt,normalized_prompt,confirmed_equivalent,created_at,last_seen_at
)
SELECT 'legacy-variant-' || id,id,prompt,normalized_key,1,created_at,last_seen_at
FROM question_bank_items;

INSERT INTO question_bank_memberships(question_id,status,added_reason,added_at)
SELECT id,'active','legacy',created_at FROM question_bank_items;

INSERT INTO question_observations(
    id,question_id,variant_id,event_type,source_type,source_id,source_item_id,
    occurred_at,verification_state,legacy_count,created_at,updated_at
)
SELECT 'legacy-observation-' || id,id,'legacy-variant-' || id,'imported_legacy',
       'legacy','question_bank_items',id,last_seen_at,'inferred',occurrence_count,created_at,updated_at
FROM question_bank_items
WHERE occurrence_count > 0;

UPDATE interview_session_questions
SET canonical_question_id = (
    SELECT cq.id
    FROM canonical_questions cq
    JOIN question_variants qv ON qv.question_id=cq.id
    WHERE qv.normalized_prompt = lower(
        replace(replace(replace(replace(replace(replace(replace(replace(
        interview_session_questions.prompt,' ',''),'，',''),'。',''),'！',''),'？',''),'?',''),',',''),'.','')
    )
    LIMIT 1
);

CREATE TRIGGER question_observations_delete_interview
AFTER DELETE ON interview_sessions
BEGIN
    DELETE FROM question_observations
    WHERE source_type='interview_session' AND source_id=OLD.id;
END;

CREATE TRIGGER question_observations_delete_experience
AFTER DELETE ON interview_experience_sources
BEGIN
    DELETE FROM question_observations
    WHERE source_type='experience' AND source_id=OLD.id;
END;
