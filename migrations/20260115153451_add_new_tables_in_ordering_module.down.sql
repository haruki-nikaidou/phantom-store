ALTER TABLE "shop"."user_order"
    DROP COLUMN tracking_number;

DROP TABLE IF EXISTS "shop"."delivery_tracking";
DROP TABLE IF EXISTS "shop"."payment_callback";

DROP TYPE IF EXISTS "shop"."delivery_status";
