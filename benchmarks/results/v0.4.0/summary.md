## Scope scope 0.1.0 Benchmark Results

**Benchmark date:** 2026-03-20
**Tasks:** 10
**Repetitions:** 1 per task per condition

### Token Consumption

| Condition | Mean input tokens | Reduction |
|-----------|-------------------|-----------|
| Without Scope | 40263 | — |
| With Scope | 36525 | **9.3%** |

### Task Correctness

| Condition | Compilation pass | Tests pass | Mean score |
|-----------|-----------------|------------|------------|
| Without Scope | 80% | 0% | 40 |
| With Scope | 80% | 0% | 40 |

### File Reads per Task

| Condition | Mean file reads |
|-----------|----------------|
| Without Scope | 9.3 |
| With Scope | 3.8 |

### By Category

| Category | With Scope (tokens) | Without Scope (tokens) | Reduction |
|----------|--------------------|-----------------------|-----------|
| cat-a | 34545 | 31875 | -8.4% |
| cat-b | 34469 | 31920 | -8.0% |
| cat-c | 38008 | 40204 | 5.5% |
| cat-d | 35584 | 42886 | 17.0% |
| cat-e | 40019 | 54432 | 26.5% |

*All results are means across 1 repetitions per task.*

### Agent Behavior Analysis

#### Navigation Efficiency
| Metric | With Scope | Without Scope |
|--------|-----------|---------------|
| Actions before first edit | 7.6 | 10.6 |
| Navigation:edit ratio | 4.6 | 7.0 |
| Unique files read | 3.7 | 9.2 |
| Redundant file reads | 0.0 | 0.0 |

#### Scope Anti-Patterns Detected
| Pattern | Count |
|---------|-------|
| Sketch then read same file | 0 |
| Grep after scope find | 0 |
| callers + refs same symbol | 0 |

#### Scope Command Usage
- Mean scope commands per task: 3.9

### CLI Recommendations

No actionable recommendations from current data.
