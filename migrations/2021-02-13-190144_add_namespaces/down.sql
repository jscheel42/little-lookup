-- This file should undo anything in `up.sql`

ALTER TABLE items DROP COLUMN IF EXISTS "namespace";