CREATE TABLE email_messages (
    id TEXT PRIMARY KEY,
    account TEXT NOT NULL,
    mailbox TEXT NOT NULL DEFAULT 'INBOX',
    uid INTEGER NOT NULL,
    message_id TEXT,
    sender TEXT NOT NULL DEFAULT '',
    subject TEXT NOT NULL DEFAULT '',
    received_at TEXT NOT NULL,
    body_text TEXT NOT NULL DEFAULT '',
    snippet TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL,
    suggested_stage TEXT,
    status TEXT NOT NULL DEFAULT 'unmatched'
        CHECK(status IN ('unmatched', 'pending', 'confirmed', 'ignored')),
    matched_application_id TEXT REFERENCES applications(id),
    confidence INTEGER NOT NULL DEFAULT 0 CHECK(confidence BETWEEN 0 AND 100),
    reasons_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(account, mailbox, uid)
);

CREATE INDEX email_messages_received_idx ON email_messages(received_at DESC);
CREATE INDEX email_messages_status_idx ON email_messages(status, received_at DESC);
CREATE INDEX email_messages_application_idx ON email_messages(matched_application_id);

CREATE TABLE email_sync_state (
    account TEXT NOT NULL,
    mailbox TEXT NOT NULL DEFAULT 'INBOX',
    last_uid INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY(account, mailbox)
);
