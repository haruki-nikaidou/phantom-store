-- Drop indexes
DROP INDEX IF EXISTS "auth"."idx_oauth_account_provider";
DROP INDEX IF EXISTS "auth"."idx_oauth_account_user_id";
DROP INDEX IF EXISTS "auth"."idx_email_otp_expires_at";
DROP INDEX IF EXISTS "auth"."idx_email_otp_email";
DROP INDEX IF EXISTS "auth"."idx_email_otp_user_id";

-- Drop tables in reverse order of creation (respecting foreign key dependencies)
DROP TABLE IF EXISTS "auth"."totp";
DROP TABLE IF EXISTS "auth"."oauth_account";
DROP TABLE IF EXISTS "auth"."email_otp";
DROP TABLE IF EXISTS "auth"."user_password";
DROP TABLE IF EXISTS "auth"."user_account";

-- Drop enum types
DROP TYPE IF EXISTS "auth"."email_otp_usage";

-- Drop schema (only if empty)
DROP SCHEMA IF EXISTS auth;
