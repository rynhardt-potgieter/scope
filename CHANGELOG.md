# Changelog

## v0.1.0 (2026-03-19)

### Features
- `sc init` -- initialise Scope for a project
- `sc index` -- build/refresh code index (incremental by default)
- `sc sketch` -- structural overview of classes, methods, interfaces, files
- `sc refs` -- find all references with kind filtering
- `sc deps` -- dependency queries with transitive depth
- `sc impact` -- blast radius analysis with recursive CTE
- `sc find` -- semantic search via SQLite FTS5
- `sc status` -- index health reporting
- TypeScript + C# language support
- `--json` output on all commands

### Known Limitations
- Edge `from_id` uses synthetic `__module__` IDs -- impact depth limited
- `sc find` uses FTS5 (keyword matching), not vector embeddings
- `sc rdeps`, `sc similar`, `sc source` not yet implemented
- No `--watch` mode
