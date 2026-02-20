-- Derived Apex class/trigger name extracted during parsing.
-- Not raw log content; safe to persist for filtering.
ALTER TABLE parsed_log_events ADD COLUMN IF NOT EXISTS class_name TEXT;

CREATE INDEX IF NOT EXISTS idx_parsed_log_events_class_name ON parsed_log_events(job_id, class_name)
  WHERE class_name IS NOT NULL;
