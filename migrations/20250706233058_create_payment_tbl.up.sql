-- Add up migration script here
CREATE TABLE IF NOT EXISTS payments(
  id INTEGER PRIMARY KEY,
  user_id INTEGER,
  order_id INTEGER,
  cost REAL NOT NULL,
  status INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);
