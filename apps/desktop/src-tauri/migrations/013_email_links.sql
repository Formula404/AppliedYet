ALTER TABLE email_messages ADD COLUMN links_json TEXT NOT NULL DEFAULT '[]';

-- Re-read the latest batch once so existing locally indexed HTML messages can
-- be enriched with href targets. UID upsert prevents duplicate records.
UPDATE email_sync_state SET last_uid=0;
