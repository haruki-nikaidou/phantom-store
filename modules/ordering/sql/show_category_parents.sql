-- Recursively fetch all ancestor categories (from direct parent up to root)
-- Results ordered from root ancestor to direct parent
WITH RECURSIVE parent_categories AS (
    -- Base case: get the direct parent of the input category
    SELECT c.id, c.name, c.parent_id, c.description, 1 AS depth
    FROM "shop"."category" c
    WHERE c.id = (SELECT parent_id FROM "shop"."category" WHERE id = $1)

    UNION ALL

    -- Recursive case: traverse up to get all ancestors
    SELECT c.id, c.name, c.parent_id, c.description, pc.depth + 1 AS depth
    FROM "shop"."category" c
    INNER JOIN parent_categories pc ON c.id = pc.parent_id
)
SELECT
    id AS "id!",
    name AS "name!",
    parent_id,
    description AS "description!"
FROM parent_categories
ORDER BY depth DESC  -- Root ancestor first, direct parent last
