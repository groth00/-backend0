-- Add up migration script here
CREATE TABLE IF NOT EXISTS inventory(
  id INTEGER PRIMARY KEY,
  pid INTEGER,
  num_available INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(pid) REFERENCES products(id)
);
