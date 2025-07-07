-- Add up migration script here
CREATE TABLE IF NOT EXISTS orders(
  id INTEGER PRIMARY KEY,
  pid INTEGER,
  num INTEGER NOT NULL,
  FOREIGN KEY(pid) REFERENCES products(id)
);
