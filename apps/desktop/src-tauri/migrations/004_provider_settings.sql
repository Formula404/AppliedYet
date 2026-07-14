CREATE TABLE provider_settings (
    provider TEXT PRIMARY KEY CHECK(provider IN ('ai', 'asr')),
    config_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
