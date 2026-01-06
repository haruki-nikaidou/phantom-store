CREATE TABLE IF NOT EXISTS "application__config"
(
    id      SERIAL PRIMARY KEY,
    key     VARCHAR(255) NOT NULL UNIQUE,
    content JSON         NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_application_key" ON "application__config" ("key");
