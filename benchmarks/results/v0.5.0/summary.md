## Scope scope 0.1.0 Benchmark Results

**Benchmark date:** 2026-03-21
**Tasks:** 10
**Repetitions:** 1 per task per condition

### Token Consumption

| Condition | Mean input tokens | Reduction |
|-----------|-------------------|-----------|
| Without Scope | 40402 | — |
| With Scope | 39558 | **2.1%** |

### Task Correctness

| Condition | Compilation pass | Tests pass | Mean score |
|-----------|-----------------|------------|------------|
| Without Scope | 80% | 0% | 40 |
| With Scope | 80% | 0% | 40 |

### File Reads per Task

| Condition | Mean file reads |
|-----------|----------------|
| Without Scope | 8.7 |
| With Scope | 3.9 |

### By Category

| Category | With Scope (tokens) | Without Scope (tokens) | Reduction |
|----------|--------------------|-----------------------|-----------|
| cat-a | 32103 | 29908 | -7.3% |
| cat-b | 31822 | 36807 | 13.5% |
| cat-c | 36070 | 36584 | 1.4% |
| cat-d | 36683 | 36659 | -0.1% |
| cat-e | 61114 | 62050 | 1.5% |

*All results are means across 1 repetitions per task.*

### Agent Behavior Analysis

#### Navigation Efficiency
| Metric | With Scope | Without Scope |
|--------|-----------|---------------|
| Actions before first edit | 7.5 | 10.5 |
| Navigation:edit ratio | 4.8 | 7.6 |
| Unique files read | 3.8 | 9.1 |
| Redundant file reads | 0.0 | 0.0 |

#### Scope Anti-Patterns Detected
| Pattern | Count |
|---------|-------|
| Sketch then read same file | 0 |
| Grep after scope find | 0 |
| callers + refs same symbol | 0 |

#### Scope Command Usage
- Mean scope commands per task: 3.7

### CLI Recommendations

Based on agent behavior data:

1. **`scope refs` never used** — agents use `scope callers` exclusively. Consider merging refs into callers or improving refs discoverability.
