-- One row per device: whole Stats JSON as an opaque snapshot.
CREATE TABLE IF NOT EXISTS devices (
  device_id  TEXT PRIMARY KEY,
  counters   TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
