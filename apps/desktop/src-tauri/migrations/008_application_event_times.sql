ALTER TABLE application_events ADD COLUMN updated_at TEXT;

UPDATE application_events SET updated_at = created_at WHERE updated_at IS NULL;

CREATE TRIGGER application_events_fill_updated_at
AFTER INSERT ON application_events
WHEN NEW.updated_at IS NULL
BEGIN
    UPDATE application_events SET updated_at = NEW.created_at WHERE id = NEW.id;
END;
