-- Add migration script here

CREATE TABLE IF NOT EXISTS "accounts" (
    "id" BIGINT PRIMARY KEY NOT NULL,
    "name" VARCHAR(32) UNIQUE NOT NULL,
    "password" VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS "repos" (
    "id" BIGINT PRIMARY KEY NOT NULL,
    "name" VARCHAR(32) UNIQUE NOT NULL,
    "owner" BIGINT REFERENCES "accounts"("id") NOT NULL,
    "owner_name" VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS "sessions" (
    "id" BIGINT PRIMARY KEY NOT NULL,
    "owner" BIGINT REFERENCES "accounts"("id") NOT NULL,
    "key" VARCHAR NOT NULL
)