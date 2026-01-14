-- Drop trigger
DROP TRIGGER IF EXISTS trigger_coupon_updated_at ON "shop"."coupon";

-- Drop trigger function
DROP FUNCTION IF EXISTS "shop"."update_updated_at_column"();

-- Drop indexes
DROP INDEX IF EXISTS "shop"."idx_user_order_is_soft_deleted";
DROP INDEX IF EXISTS "shop"."idx_user_order_created_at";
DROP INDEX IF EXISTS "shop"."idx_user_order_status";
DROP INDEX IF EXISTS "shop"."idx_user_order_user";
DROP INDEX IF EXISTS "shop"."idx_coupon_set_active";
DROP INDEX IF EXISTS "shop"."idx_coupon_code";
DROP INDEX IF EXISTS "shop"."idx_goods_on_sale";
DROP INDEX IF EXISTS "shop"."idx_goods_category_id";
DROP INDEX IF EXISTS "shop"."idx_category_parent_id";

-- Drop tables in reverse order of creation (respecting foreign key dependencies)
DROP TABLE IF EXISTS "shop"."user_order";
DROP TABLE IF EXISTS "shop"."coupon";
DROP TABLE IF EXISTS "shop"."goods";
DROP TABLE IF EXISTS "shop"."category";

-- Drop enum types
DROP TYPE IF EXISTS "shop"."payment_method";
DROP TYPE IF EXISTS "shop"."order_status";

-- Drop schema (only if empty)
DROP SCHEMA IF EXISTS shop;
