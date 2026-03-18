# TypeScript Simple Fixture

Small TypeScript project used as the ground truth for integration and snapshot tests.
Never edit these files without rebuilding the index and updating all affected tests.

## Project Structure

```
src/
  payments/
    service.ts      # PaymentService class — 4 methods (constructor, processPayment,
                    #   refundPayment, validateAmount) + 1 field (logger)
    types.ts        # PaymentRequest interface, PaymentResult interface, PaymentStatus type
  controllers/
    order.ts        # OrderController — imports PaymentService, calls processPayment twice
    refund.ts       # RefundController — imports PaymentService, calls refundPayment once
  utils/
    logger.ts       # Logger class — 2 methods (info, error)
tsconfig.json
```

## Index Ground Truth

These numbers are what tests assert against. Verified by running `scope index --full` followed
by the queries below.

### Symbol Counts

| Metric | Value |
|--------|-------|
| Total files indexed | 5 |
| Total symbols | 21 |
| Total edges | 17 |

### `scope sketch PaymentService`

- Kind: `class`
- File: `src/payments/service.ts:4-24`
- Methods: `constructor`, `processPayment`, `refundPayment`, `validateAmount` (4 methods)
- Fields: `private logger: Logger` (1 field)
- All methods show `[internal]` caller count (no external callers in the fixture)

### `scope refs PaymentService`

- Total references: **4**
- `imported` (2): `src/controllers/order.ts:1`, `src/controllers/refund.ts:1`
- `used as type` (2): `src/controllers/order.ts:5`, `src/controllers/refund.ts:4`

### `scope refs processPayment`

- Total references: **0** (calls are on the paymentService field, not the class method directly)

### `scope refs Logger`

- Total references: **2**
- `imported` (1): `src/payments/service.ts:2`
- `used as type` (1): `src/payments/service.ts:5`

### `scope deps PaymentService`

Groups:
- `imports`: Logger (logger.ts), PaymentRequest (types.ts), PaymentResult (types.ts)
- `references_type`: Logger (logger.ts), PaymentRequest (types.ts)

### `scope impact PaymentService`

- Result: `(no impact detected)` — no reverse-call edges reach PaymentService in this fixture.

### `scope find "payment"`

Returns results in score order, top entries:
1. `processPayment` — score `1.00` — method
2. `refundPayment` — score `1.00` — method
3. `PaymentService` — score `0.93` — class
4. `PaymentRequest` — score `0.93` — interface
5. `PaymentResult` — score `0.93` — interface

### `scope sketch Logger`

- Kind: `class`
- File: `src/utils/logger.ts:1-9`
- Methods: `info`, `error` (2 methods)

### `scope sketch PaymentRequest`

- Kind: `interface`
- File: `src/payments/types.ts:1-5`

## Rebuilding the Index

Run from the `tests/fixtures/typescript-simple/` directory:

```bash
scope init    # only if .scope/ does not already exist
scope index --full
```

After a schema change commit the new `.scope/graph.db` and `.scope/file_hashes.db`.
