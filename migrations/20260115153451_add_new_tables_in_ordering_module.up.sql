ALTER TABLE "shop"."user_order"
    ADD COLUMN tracking_number TEXT;

CREATE TYPE "shop"."delivery_status" AS ENUM (
    'in_transit',
    'out_for_delivery',
    'delivered',
    'cancelled',
    'returned'
    );

CREATE TABLE IF NOT EXISTS "shop"."delivery_tracking"
(
    id          BIGSERIAL PRIMARY KEY,
    order_id    UUID                     NOT NULL REFERENCES "shop"."user_order" (id) ON DELETE CASCADE,
    status      "shop"."delivery_status" NOT NULL,
    location    TEXT,
    description TEXT                     NOT NULL,
    created_at  TIMESTAMP                NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_delivery_tracking_order_id ON "shop"."delivery_tracking" (order_id);

CREATE TABLE IF NOT EXISTS "shop"."payment_callback"
(
    id                  BIGSERIAL PRIMARY KEY,
    order_id            UUID                    NOT NULL REFERENCES "shop"."user_order" (id) ON DELETE CASCADE,
    payment_method      "shop"."payment_method" NOT NULL,
    payment_method_info JSONB                   NOT NULL,
    created_at          TIMESTAMP               NOT NULL DEFAULT NOW(),
    checked_at          TIMESTAMP
);