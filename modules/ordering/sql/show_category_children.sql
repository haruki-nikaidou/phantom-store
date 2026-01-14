-- Fetch all direct children of the given category
SELECT id, name, parent_id, description
FROM "shop"."category"
WHERE parent_id = $1
ORDER BY id
