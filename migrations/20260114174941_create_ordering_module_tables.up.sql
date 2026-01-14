-- Create shop schema for ordering module
CREATE SCHEMA IF NOT EXISTS shop;

-- Create order_status enum type
CREATE TYPE "shop"."order_status" AS ENUM (
    'unpaid',
    'paid',
    'delivered',
    'arrived',
    'cancelled',
    'refunding',
    'refunded'
);

-- Create payment_method enum type
CREATE TYPE "shop"."payment_method" AS ENUM (
    'stable_coin',
    'credit_card',
    'pay_pal',
    'admin_operation'
);

-- Create category table
CREATE TABLE IF NOT EXISTS "shop"."category"
(
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    parent_id   INTEGER REFERENCES "shop"."category" (id) ON DELETE CASCADE,
    description TEXT         NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_category_parent_id ON "shop"."category" (parent_id);

-- Create goods table
CREATE TABLE IF NOT EXISTS "shop"."goods"
(
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255)   NOT NULL,
    description TEXT           NOT NULL DEFAULT '',
    pictures    TEXT[]         NOT NULL DEFAULT '{}',
    price       DECIMAL(19, 4) NOT NULL,
    category_id INTEGER REFERENCES "shop"."category" (id) ON DELETE RESTRICT,
    on_sale     BOOLEAN        NOT NULL DEFAULT TRUE,
    stock       INTEGER        NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_goods_category_id ON "shop"."goods" (category_id);
CREATE INDEX IF NOT EXISTS idx_goods_on_sale ON "shop"."goods" (on_sale);

-- Create coupon table
CREATE TABLE IF NOT EXISTS "shop"."coupon"
(
    id                SERIAL PRIMARY KEY,
    code              VARCHAR(255) NOT NULL UNIQUE,
    set_active        BOOLEAN      NOT NULL DEFAULT TRUE,
    discount          JSONB        NOT NULL,
    available_since   TIMESTAMP,
    available_until   TIMESTAMP,
    limit_to_category INTEGER REFERENCES "shop"."category" (id) ON DELETE SET NULL,
    limit_per_user    INTEGER,
    limit_total       INTEGER,
    used_count        INTEGER      NOT NULL DEFAULT 0,
    created_at        TIMESTAMP    NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMP    NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_coupon_code ON "shop"."coupon" (code);
CREATE INDEX IF NOT EXISTS idx_coupon_set_active ON "shop"."coupon" (set_active);

-- Create user_order table
CREATE TABLE IF NOT EXISTS "shop"."user_order"
(
    id                  UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    "user"              UUID                           NOT NULL REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    production          UUID                           NOT NULL,
    total_amount        DECIMAL(19, 4)                 NOT NULL,
    coupon_used         INTEGER REFERENCES "shop"."coupon" (id) ON DELETE RESTRICT,
    created_at          TIMESTAMP                      NOT NULL DEFAULT NOW(),
    order_status        "shop"."order_status"          NOT NULL DEFAULT 'unpaid',

    paid_at             TIMESTAMP,
    delivered_at        TIMESTAMP,
    arrived_at          TIMESTAMP,
    cancelled_at        TIMESTAMP,
    refund_requested_at TIMESTAMP,
    refunded_at         TIMESTAMP,

    payment_method      "shop"."payment_method",
    payment_method_info JSONB                          NOT NULL DEFAULT '{}',

    is_soft_deleted     BOOLEAN                        NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_user_order_user ON "shop"."user_order" ("user");
CREATE INDEX IF NOT EXISTS idx_user_order_status ON "shop"."user_order" (order_status);
CREATE INDEX IF NOT EXISTS idx_user_order_created_at ON "shop"."user_order" (created_at);
CREATE INDEX IF NOT EXISTS idx_user_order_is_soft_deleted ON "shop"."user_order" (is_soft_deleted);

-- Create trigger function to auto-update updated_at column
CREATE OR REPLACE FUNCTION "shop"."update_updated_at_column"()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for coupon table
CREATE TRIGGER trigger_coupon_updated_at
    BEFORE UPDATE ON "shop"."coupon"
    FOR EACH ROW
    EXECUTE FUNCTION "shop"."update_updated_at_column"();
