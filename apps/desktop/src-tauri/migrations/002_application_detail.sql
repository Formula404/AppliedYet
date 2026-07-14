ALTER TABLE tasks ADD COLUMN application_stage TEXT;
ALTER TABLE tasks ADD COLUMN deleted_at TEXT;

CREATE INDEX tasks_application_idx
    ON tasks(application_id, status, due_at) WHERE deleted_at IS NULL;
