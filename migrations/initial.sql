CREATE TABLE IF NOT EXISTS Configuration (
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL, -- 'grade'|'stage'
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ProductTemplates (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  specs_json TEXT NOT NULL
);

-- Inventory table uses a JSON column to store dynamic spec key/values
CREATE TABLE IF NOT EXISTS inventory_batches (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  type_id INTEGER,
  grade_id INTEGER,
  stage_id INTEGER,
  weight REAL NOT NULL,
  price REAL NOT NULL,
  specs_json TEXT NOT NULL,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
