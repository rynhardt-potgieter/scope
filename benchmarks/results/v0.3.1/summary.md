## Scope scope 0.1.0 Benchmark Results

**Benchmark date:** 2026-03-20
**Tasks:** 10
**Repetitions:** 1 per task per condition

### Token Consumption

| Condition | Mean input tokens | Reduction |
|-----------|-------------------|-----------|
| Without Scope | 36035 | — |
| With Scope | 30292 | **15.9%** |

### Task Correctness

| Condition | Compilation pass | Tests pass | Mean score |
|-----------|-----------------|------------|------------|
| Without Scope | 80% | 0% | 40 |
| With Scope | 80% | 0% | 40 |

### File Reads per Task

| Condition | Mean file reads |
|-----------|----------------|
| Without Scope | 9.6 |
| With Scope | 4.5 |

### By Category

| Category | With Scope (tokens) | Without Scope (tokens) | Reduction |
|----------|--------------------|-----------------------|-----------|
| cat-a | 24677 | 29692 | 16.9% |
| cat-b | 28110 | 30258 | 7.1% |
| cat-c | 31512 | 33196 | 5.1% |
| cat-d | 33368 | 35683 | 6.5% |
| cat-e | 33792 | 51346 | 34.2% |

*All results are means across 1 repetitions per task.*

### Agent Behavior Analysis

#### Navigation Efficiency
| Metric | With Scope | Without Scope |
|--------|-----------|---------------|
| Actions before first edit | 7.6 | 11.0 |
| Navigation:edit ratio | 5.9 | 7.6 |
| Unique files read | 3.9 | 9.6 |
| Redundant file reads | 0.0 | 0.0 |

#### Scope Anti-Patterns Detected
| Pattern | Count |
|---------|-------|
| Sketch then read same file | 0 |
| Grep after scope find | 0 |
| callers + refs same symbol | 0 |

#### Scope Command Usage
- Mean scope commands per task: 3.6

### CLI Recommendations

Based on agent behavior data:

1. **`scope refs` never used** — agents use `scope callers` exclusively. Consider merging refs into callers or improving refs discoverability.
