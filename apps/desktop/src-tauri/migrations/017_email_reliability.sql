ALTER TABLE email_sync_state ADD COLUMN uid_validity INTEGER;

CREATE TABLE email_sync_failures (
    account TEXT NOT NULL,
    mailbox TEXT NOT NULL DEFAULT 'INBOX',
    uid INTEGER NOT NULL,
    reason TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 1,
    permanently_skipped INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY(account, mailbox, uid)
);

CREATE INDEX email_sync_failures_retry_idx
    ON email_sync_failures(permanently_skipped, last_attempt_at);
