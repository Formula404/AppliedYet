UPDATE application_events
SET recorded_at = COALESCE(updated_at, created_at)
WHERE recorded_at IS NULL;

DROP TRIGGER IF EXISTS application_events_fill_updated_at;

CREATE TRIGGER application_events_fill_times
AFTER INSERT ON application_events
WHEN NEW.updated_at IS NULL OR NEW.recorded_at IS NULL
BEGIN
    UPDATE application_events
    SET updated_at = COALESCE(NEW.updated_at, NEW.created_at),
        recorded_at = COALESCE(NEW.recorded_at, NEW.created_at)
    WHERE id = NEW.id;
END;

CREATE INDEX IF NOT EXISTS application_events_drag_merge_idx
ON application_events(application_id, source_id, recorded_at DESC)
WHERE event_type = 'stage_changed' AND reverted_at IS NULL;

-- 修复旧版本并发拖拽可能留下的 current_stage / 时间线不一致。
UPDATE applications
SET current_stage = COALESCE(
        (SELECT e.stage_after FROM application_events e
         WHERE e.application_id = applications.id AND e.stage_after IS NOT NULL
           AND e.reverted_at IS NULL AND e.event_type <> 'event_reverted'
         ORDER BY e.happened_at DESC, e.created_at DESC, e.rowid DESC LIMIT 1),
        current_stage
    ),
    status_updated_at = COALESCE(
        (SELECT e.happened_at FROM application_events e
         WHERE e.application_id = applications.id AND e.stage_after IS NOT NULL
           AND e.reverted_at IS NULL AND e.event_type <> 'event_reverted'
         ORDER BY e.happened_at DESC, e.created_at DESC, e.rowid DESC LIMIT 1),
        status_updated_at
    )
WHERE deleted_at IS NULL;
