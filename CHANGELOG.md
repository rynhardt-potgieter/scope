# Changelog

## v0.1.0 (2026-03-19)

### Features
- `scope init` -- initialise Scope for a project
- `scope index` -- build/refresh code index (incremental by default)
- `scope sketch` -- structural overview of classes, methods, interfaces, files
- `scope refs` -- find all references with kind filtering
- `scope deps` -- dependency queries with transitive depth
- `scope impact` -- blast radius analysis with recursive CTE
- `scope find` -- semantic search via SQLite FTS5
- `scope status` -- index health reporting
- TypeScript + C# language support
- `--json` output on all commands

### Known Limitations
- Edge `from_id` uses synthetic `__module__` IDs -- impact depth limited
- `scope find` uses FTS5 (keyword matching), not vector embeddings
- `scope rdeps`, `scope similar`, `scope source` not yet implemented
- No `--watch` mode
