CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE SCHEMA IF NOT EXISTS admin;

CREATE TYPE "admin"."admin_role" AS ENUM (
    'owner',
    'moderator'
    );

CREATE TABLE IF NOT EXISTS "admin"."admin_account"
(
    id            UUID PRIMARY KEY              DEFAULT gen_random_uuid(),
    role          "admin"."admin_role" NOT NULL,
    name          TEXT                 NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ          NOT NULL DEFAULT NOW(),
    password_hash TEXT                 NOT NULL,
    email         VARCHAR(255)         NOT NULL UNIQUE,
    avatar        TEXT
);