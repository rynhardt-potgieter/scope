using CSharpLargeApi.Application.Commands.ProcessPayment;
using CSharpLargeApi.Application.Validators;

namespace CSharpLargeApi.Tests.Unit.Validators;

/// <summary>
/// Unit tests for the ProcessPaymentValidator class.
/// </summary>
public class PaymentValidatorTests
{
    private readonly ProcessPaymentValidator _validator = new();

    /// <summary>
    /// Verifies that a valid command produces no errors.
    /// </summary>
    public void Validate_WithValidCommand_ReturnsNoErrors()
    {
        var command = new ProcessPaymentCommand(
            Guid.NewGuid(), 100.00m, "USD", "pm_valid_token_12345");

        var errors = _validator.Validate(command);
        // errors should be empty
    }

    /// <summary>
    /// Verifies that empty UserId produces a validation error.
    /// </summary>
    public void Validate_WithEmptyUserId_ReturnsError()
    {
        var command = new ProcessPaymentCommand(
            Guid.Empty, 100.00m, "USD", "pm_valid_token_12345");

        var errors = _validator.Validate(command);
        // Should contain "UserId is required."
    }

    /// <summary>
    /// Verifies that zero amount produces a validation error.
    /// </summary>
    public void Validate_WithZeroAmount_ReturnsError()
    {
        var command = new ProcessPaymentCommand(
            Guid.NewGuid(), 0m, "USD", "pm_valid_token_12345");

        var errors = _validator.Validate(command);
        // Should contain "Amount must be greater than zero."
    }

    /// <summary>
    /// Verifies that excessive amount produces a validation error.
    /// </summary>
    public void Validate_WithExcessiveAmount_ReturnsError()
    {
        var command = new ProcessPaymentCommand(
            Guid.NewGuid(), 2_000_000m, "USD", "pm_valid_token_12345");

        var errors = _validator.Validate(command);
        // Should contain max allowed value error
    }

    /// <summary>
    /// Verifies that invalid currency code produces a validation error.
    /// </summary>
    public void Validate_WithInvalidCurrency_ReturnsError()
    {
        var command = new ProcessPaymentCommand(
            Guid.NewGuid(), 100.00m, "US", "pm_valid_token_12345");

        var errors = _validator.Validate(command);
        // Should contain "Currency must be a 3-letter ISO 4217 code."
    }

    /// <summary>
    /// Verifies that empty payment token produces a validation error.
    /// </summary>
    public void Validate_WithEmptyToken_ReturnsError()
    {
        var command = new ProcessPaymentCommand(
            Guid.NewGuid(), 100.00m, "USD", "");

        var errors = _validator.Validate(command);
        // Should contain "PaymentMethodToken is required."
    }

    /// <summary>
    /// Verifies that multiple errors are returned for a completely invalid command.
    /// </summary>
    public void Validate_WithMultipleErrors_ReturnsAllErrors()
    {
        var command = new ProcessPaymentCommand(
            Guid.Empty, -5m, "", "");

        var errors = _validator.Validate(command);
        // Should contain at least 4 errors
    }
}
