-- Remove raw log content that should never have been persisted (privacy requirement).
ALTER TABLE parsed_log_events DROP COLUMN IF EXISTS raw_line;
ALTER TABLE parsed_log_events DROP COLUMN IF EXISTS message;

-- 30-day rolling retention: jobs auto-expire and CASCADE deletes events + benchmarks.
ALTER TABLE parse_jobs ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;
UPDATE parse_jobs SET expires_at = created_at + INTERVAL '30 days' WHERE expires_at IS NULL;
ALTER TABLE parse_jobs ALTER COLUMN expires_at SET DEFAULT NOW() + INTERVAL '30 days';
ALTER TABLE parse_jobs ALTER COLUMN expires_at SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_parse_jobs_expires_at ON parse_jobs(expires_at);
