## Scope scope 0.1.0 Benchmark Results

**Benchmark date:** 2026-03-21
**Tasks:** 12
**Repetitions:** 3 per task per condition

### Token Consumption

| Condition | Mean input tokens | Std Dev | Reduction |
|-----------|-------------------|---------|-----------|
| Without Scope | 38119 | ±7802 | — |
| With Scope | 35702 | ±8172 | **6.3%** |

### Task Correctness

| Condition | Compilation pass | Tests pass | Mean score |
|-----------|-----------------|------------|------------|
| Without Scope | 100% | 0% | 50 |
| With Scope | 100% | 0% | 50 |

### File Reads per Task

| Condition | Mean file reads |
|-----------|----------------|
| Without Scope | 6.9 |
| With Scope | 3.8 |

### By Category

| Category | With Scope (tokens) | Without Scope (tokens) | Reduction |
|----------|--------------------|-----------------------|-----------|
| cat-a | 29888 | 35102 | 14.9% |
| cat-b | 30643 | 30755 | 0.4% |
| cat-c | 36864 | 40120 | 8.1% |
| cat-d | 32662 | 37540 | 13.0% |
| cat-e | 40154 | 42766 | 6.1% |
| cat-f | 43998 | 42432 | -3.7% |

*All results are means across 3 repetitions per task.*
