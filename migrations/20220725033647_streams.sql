-- Add migration script here
CREATE TABLE IF NOT EXISTS streams
(
    id INTEGER PRIMARY KEY NOT NULL,
    stream TEXT NOT NULL,
    position INTEGER NOT NULL,
    data TEXT NOT NULL,
    UNIQUE(stream, position)
);
