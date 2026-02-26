CREATE TABLE audit_events (
  id         BIGSERIAL PRIMARY KEY,
  event_type TEXT NOT NULL,
  actor_user_id  BIGINT REFERENCES app_users(id) ON DELETE SET NULL,
  actor_ip       TEXT,
  target_user_id BIGINT REFERENCES app_users(id) ON DELETE SET NULL,
  org_id         BIGINT REFERENCES organizations(id) ON DELETE SET NULL,
  metadata       JSONB NOT NULL DEFAULT '{}',
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_events_event_type ON audit_events (event_type);
CREATE INDEX idx_audit_events_actor      ON audit_events (actor_user_id);
CREATE INDEX idx_audit_events_org        ON audit_events (org_id);
CREATE INDEX idx_audit_events_created    ON audit_events (created_at);
