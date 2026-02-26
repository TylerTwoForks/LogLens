-- Add password_hash column to support password-based authentication.
-- Nullable so existing email-only users continue to work during transition.
ALTER TABLE app_users ADD COLUMN password_hash TEXT;
