DROP TABLE lookup_config;
CREATE TABLE lookup_config (
  code TEXT PRIMARY KEY,                  -- mis: 'countries', 'banks'
  table_name TEXT NOT NULL,               -- mis: 'country', 'mst_banks'
  display_cols TEXT[] NOT NULL,           -- kolom yang ditampilkan di frontend
  searchable_col TEXT,                    -- kolom untuk filter (autocomplete)
  label TEXT NOT NULL,                    -- deskripsi/label frontend
  mode TEXT NOT NULL DEFAULT 'dropdown'   -- dropdown, autocomplete, tree, etc
);

INSERT INTO lookup_config (
  code,
  table_name,
  display_cols,
  searchable_col,
  label,
  mode
)
VALUES (
  'countries',
  'country',
  ARRAY['country_id', 'iso_code', 'country_name'],
  'country_name',
  'countries',
  'autocomplete'
);
