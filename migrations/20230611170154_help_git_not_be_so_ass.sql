-- Add migration script here

CREATE TABLE IF NOT EXISTS "git_commit" (
    "id" BIGSERIAL PRIMARY KEY NOT NULL,
    "oid" VARCHAR UNIQUE NOT NULL,
    "inserted_at" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    -- Or, maybe, in the future, idk, we could, maybe, use the id field, who knows, maybe
    "parent" VARCHAR REFERENCES "git_commit"("oid")
);

CREATE INDEX git_commit_oid ON "git_commit"("oid");

CREATE TABLE IF NOT EXISTS "git_object" (
    "id" BIGSERIAL PRIMARY KEY NOT NULL,
    "oid" VARCHAR UNIQUE NOT NULL,
    "inserted_at" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    "latest_commit" VARCHAR REFERENCES "git_commit"("oid") NOT NULL,
    "original_commit" VARCHAR REFERENCES "git_commit"("oid") NOT NULL
);