-- Scope graph schema
-- This file is the single source of truth for the SQLite schema.
-- Loaded via include_str! in src/core/graph.rs.

-- symbols: every named code construct
CREATE TABLE IF NOT EXISTS symbols (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK(kind IN (
                    'function','class','method','interface',
                    'struct','enum','const','type','property'
                )),
    file_path   TEXT NOT NULL,
    line_start  INTEGER NOT NULL,
    line_end    INTEGER NOT NULL,
    signature   TEXT,
    docstring   TEXT,
    parent_id   TEXT REFERENCES symbols(id) ON DELETE CASCADE,
    language    TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);

-- edges: relationships between symbols.
-- from_id and to_id are intentionally NOT foreign keys — edges may reference
-- synthetic IDs (e.g. __module__), external library symbols, or cross-file
-- symbols that are indexed separately. Deletion is handled in insert_file_data
-- by deleting all edges WHERE file_path = ? before re-inserting.
CREATE TABLE IF NOT EXISTS edges (
    from_id     TEXT NOT NULL,
    to_id       TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK(kind IN (
                    'calls','imports','extends','implements',
                    'instantiates','references','references_type'
                )),
    file_path   TEXT NOT NULL,
    line        INTEGER,
    PRIMARY KEY (from_id, to_id, kind)
);

-- file_hashes: tracks which files are indexed and whether they have changed
CREATE TABLE IF NOT EXISTS file_hashes (
    file_path   TEXT PRIMARY KEY,
    hash        TEXT NOT NULL,
    indexed_at  INTEGER NOT NULL
);

-- Covering indices for common query patterns
CREATE INDEX IF NOT EXISTS idx_symbols_name     ON symbols(name);
CREATE INDEX IF NOT EXISTS idx_symbols_file     ON symbols(file_path);
CREATE INDEX IF NOT EXISTS idx_symbols_kind     ON symbols(kind);
CREATE INDEX IF NOT EXISTS idx_edges_from       ON edges(from_id, kind);
CREATE INDEX IF NOT EXISTS idx_edges_to         ON edges(to_id, kind);
CREATE INDEX IF NOT EXISTS idx_edges_file       ON edges(file_path);
