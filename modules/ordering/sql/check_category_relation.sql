SELECT
    COALESCE(
        (SELECT parent_id IS NOT NULL FROM "shop"."category" WHERE id = $1),
        false
    ) AS "has_parent!: bool",
    EXISTS(
        SELECT 1 FROM "shop"."category" WHERE parent_id = $1
    ) AS "has_children!: bool",
    EXISTS(
        SELECT 1 FROM "shop"."goods" WHERE category_id = $1
    ) AS "has_goods!: bool",
    EXISTS(
        SELECT 1 FROM "shop"."coupon" WHERE limit_to_category = $1
    ) AS "has_coupons!: bool"
