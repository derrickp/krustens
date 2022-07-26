-- Add migration script here
CREATE TABLE IF NOT EXISTS snapshots
(
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT UNIQUE,
    version INTEGER NOT NULL,
    data TEXT NOT NULL
);
