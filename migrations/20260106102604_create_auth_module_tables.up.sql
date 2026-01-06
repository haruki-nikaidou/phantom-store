-- Create auth schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Create email_otp_usage enum type
CREATE TYPE "auth"."email_otp_usage" AS ENUM (
    'login',
    'password_reset',
    'change_email_address',
    'sudo_mode'
);

-- Create user_account table (main user table)
CREATE TABLE IF NOT EXISTS "auth"."user_account"
(
    id         UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    name       TEXT,
    email      VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP    NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP    NOT NULL DEFAULT NOW()
);

-- Create user_password table (stores password hashes)
CREATE TABLE IF NOT EXISTS "auth"."user_password"
(
    user_id       UUID PRIMARY KEY REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    password_hash TEXT      NOT NULL,
    created_at    TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create email_otp table (stores email OTP codes)
CREATE TABLE IF NOT EXISTS "auth"."email_otp"
(
    id            BIGSERIAL PRIMARY KEY,
    user_id       UUID REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    email         VARCHAR(255)              NOT NULL,
    otp_code      VARCHAR(6)                NOT NULL,
    created_at    TIMESTAMP                 NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMP                 NOT NULL,
    has_been_used BOOLEAN                   NOT NULL DEFAULT FALSE,
    used_at       TIMESTAMP,
    usage         "auth"."email_otp_usage"  NOT NULL
);

-- Create oauth_account table (stores OAuth provider accounts)
CREATE TABLE IF NOT EXISTS "auth"."oauth_account"
(
    id               BIGSERIAL PRIMARY KEY,
    user_id          UUID         NOT NULL REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    provider_name    TEXT         NOT NULL,
    provider_user_id TEXT         NOT NULL,
    registered_at    TIMESTAMP    NOT NULL DEFAULT NOW(),
    token_updated_at TIMESTAMP    NOT NULL DEFAULT NOW(),
    UNIQUE (provider_name, provider_user_id)
);

-- Create totp table (stores TOTP secrets for 2FA)
CREATE TABLE IF NOT EXISTS "auth"."totp"
(
    user_id    UUID PRIMARY KEY REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    secret     BYTEA     NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_email_otp_user_id ON "auth"."email_otp" (user_id);
CREATE INDEX IF NOT EXISTS idx_email_otp_email ON "auth"."email_otp" (email);
CREATE INDEX IF NOT EXISTS idx_email_otp_expires_at ON "auth"."email_otp" (expires_at);
CREATE INDEX IF NOT EXISTS idx_oauth_account_user_id ON "auth"."oauth_account" (user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_account_provider ON "auth"."oauth_account" (provider_name, provider_user_id);

-- Create trigger function to auto-update updated_at column
CREATE OR REPLACE FUNCTION "auth"."update_updated_at_column"()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for tables with updated_at column
CREATE TRIGGER trigger_user_account_updated_at
    BEFORE UPDATE ON "auth"."user_account"
    FOR EACH ROW
    EXECUTE FUNCTION "auth"."update_updated_at_column"();

CREATE TRIGGER trigger_user_password_updated_at
    BEFORE UPDATE ON "auth"."user_password"
    FOR EACH ROW
    EXECUTE FUNCTION "auth"."update_updated_at_column"();

