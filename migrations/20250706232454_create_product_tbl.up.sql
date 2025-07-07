-- Add up migration script here
CREATE TABLE IF NOT EXISTS products(
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
