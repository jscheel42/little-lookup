-- Your SQL goes here

ALTER TABLE items ADD COLUMN "namespace" TEXT DEFAULT 'default' NOT NULL;

CREATE INDEX items_idx_key_namespace
ON items(key, namespace);
