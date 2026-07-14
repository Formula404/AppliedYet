ALTER TABLE tasks ADD COLUMN reminder_notified_at TEXT;

CREATE INDEX tasks_pending_reminder_idx
    ON tasks(remind_at, reminder_notified_at)
    WHERE deleted_at IS NULL AND status IN ('todo', 'doing') AND remind_at IS NOT NULL;
