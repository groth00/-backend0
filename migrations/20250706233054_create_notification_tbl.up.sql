-- Add up migration script here
CREATE TABLE IF NOT EXISTS notifications(
  id INTEGER PRIMARY KEY,
  user_id INTEGER,
  ts TEXT NOT NULL,
  msg TEXT NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id)
);
