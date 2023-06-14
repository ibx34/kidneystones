-- Add migration script here

ALTER TABLE "repos" 
ADD COLUMN "anonymous"  BOOLEAN DEFAULT TRUE, 
ADD COLUMN "creator_ip" INET;