# C# Simple Fixture

Small C# project used as the ground truth for C#-specific integration and snapshot tests.
Never edit these files without rebuilding the index and updating all affected tests.

## Project Structure

```
src/
  Payments/
    PaymentService.cs   # PaymentService class — implements IPaymentService,
                        #   4 methods (constructor, ProcessPayment, RefundPayment, ValidateAmount)
    IPaymentService.cs  # IPaymentService interface — 2 method declarations
  Controllers/
    OrderController.cs  # OrderController — uses IPaymentService, 2 methods (constructor, Checkout)
  Utils/
    Logger.cs           # Logger class — 2 methods (Info, Error)
CSharpSimple.csproj
```

## Index Ground Truth

These numbers are what tests assert against. Verified by running `scope index --full` followed
by the queries below.

### Symbol Counts

| Metric | Value |
|--------|-------|
| Total files indexed | 4 |
| Total symbols | 14 |
| Total edges | 6 |

### `scope sketch PaymentService`

- Kind: `class`
- File: `src/Payments/PaymentService.cs:5-30`
- Implements: `IPaymentService`
- Methods: `PaymentService` (constructor), `ProcessPayment`, `RefundPayment`, `ValidateAmount` (4 methods)
- All methods show `[internal]` caller count

### `scope refs PaymentService`

- Total references: **0** — OrderController depends on `IPaymentService` (the interface), not the concrete class.

### `scope refs IPaymentService`

- Total references: **1**
- `implemented` (1): `src/Payments/PaymentService.cs:5`

### `scope refs Logger`

- Total references: **0** — Logger usage via `_logger.Info` is tracked as an external call edge, not a direct symbol reference.

### `scope deps PaymentService`

Groups:
- `calls (external)`: `_logger.Info` (external)
- `imports (external)`: `CSharpSimple.Utils` (external)

### `scope sketch IPaymentService`

- Kind: `interface`
- File: `src/Payments/IPaymentService.cs:3-7`
- Methods: `ProcessPayment`, `RefundPayment` (2 method declarations)

### `scope sketch Logger`

- Kind: `class`
- File: `src/Utils/Logger.cs:3-7`
- Methods: `Info`, `Error` (2 methods)

### `scope sketch OrderController`

- Kind: `class`
- File: `src/Controllers/OrderController.cs:5-18`
- Methods: `OrderController` (constructor), `Checkout` (2 methods)

## Rebuilding the Index

Run from the `tests/fixtures/csharp-simple/` directory:

```bash
scope init    # only if .scope/ does not already exist
scope index --full
```

After a schema change commit the new `.scope/graph.db` and `.scope/file_hashes.db`.
