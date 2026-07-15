-- v11 was briefly shipped without this cursor table. Keep this repair idempotent
-- so databases that already received the final v11 schema are also valid.
CREATE TABLE IF NOT EXISTS email_sync_state (
    account TEXT NOT NULL,
    mailbox TEXT NOT NULL DEFAULT 'INBOX',
    last_uid INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY(account, mailbox)
);
