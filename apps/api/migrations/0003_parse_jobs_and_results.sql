CREATE TABLE IF NOT EXISTS parse_jobs (
  id BIGSERIAL PRIMARY KEY,
  org_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  file_name TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'done', 'failed')) DEFAULT 'queued',
  total_lines BIGINT NOT NULL DEFAULT 0,
  parsed_lines BIGINT NOT NULL DEFAULT 0,
  benchmark_count INTEGER NOT NULL DEFAULT 0,
  error_message TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  started_at TIMESTAMPTZ,
  finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_parse_jobs_org_id ON parse_jobs(org_id);
CREATE INDEX IF NOT EXISTS idx_parse_jobs_status ON parse_jobs(status);

CREATE TABLE IF NOT EXISTS parsed_log_events (
  id BIGSERIAL PRIMARY KEY,
  job_id BIGINT NOT NULL REFERENCES parse_jobs(id) ON DELETE CASCADE,
  line_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL DEFAULT '',
  nanos BIGINT,
  event_type TEXT NOT NULL DEFAULT '',
  line_number INTEGER,
  log_level TEXT,
  message TEXT NOT NULL DEFAULT '',
  raw_line TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_parsed_log_events_job_id ON parsed_log_events(job_id);
CREATE INDEX IF NOT EXISTS idx_parsed_log_events_event_type ON parsed_log_events(job_id, event_type);
CREATE INDEX IF NOT EXISTS idx_parsed_log_events_log_level ON parsed_log_events(job_id, log_level);

CREATE TABLE IF NOT EXISTS benchmark_snapshots (
  id BIGSERIAL PRIMARY KEY,
  job_id BIGINT NOT NULL REFERENCES parse_jobs(id) ON DELETE CASCADE,
  sequence INTEGER NOT NULL,
  label TEXT NOT NULL,
  query_rows BIGINT NOT NULL DEFAULT 0,
  query_rows_limit BIGINT NOT NULL DEFAULT 0,
  query_rows_delta BIGINT NOT NULL DEFAULT 0,
  heap_size_pct DOUBLE PRECISION NOT NULL DEFAULT 0.0,
  heap_size_bytes_limit BIGINT NOT NULL DEFAULT 0,
  heap_size_delta DOUBLE PRECISION NOT NULL DEFAULT 0.0,
  cpu_time_ms BIGINT NOT NULL DEFAULT 0,
  cpu_time_limit BIGINT NOT NULL DEFAULT 0,
  cpu_time_delta BIGINT NOT NULL DEFAULT 0,
  dml_statements BIGINT NOT NULL DEFAULT 0,
  dml_statements_limit BIGINT NOT NULL DEFAULT 0,
  soql_queries BIGINT NOT NULL DEFAULT 0,
  soql_queries_limit BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_benchmark_snapshots_job_id ON benchmark_snapshots(job_id);
CREATE INDEX IF NOT EXISTS idx_benchmark_snapshots_sequence ON benchmark_snapshots(job_id, sequence);
