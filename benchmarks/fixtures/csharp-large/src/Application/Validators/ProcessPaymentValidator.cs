using CSharpLargeApi.Application.Commands.ProcessPayment;

namespace CSharpLargeApi.Application.Validators;

/// <summary>
/// Validates ProcessPaymentCommand instances before they reach the handler.
/// Ensures all required fields are present and within acceptable ranges.
/// </summary>
public class ProcessPaymentValidator
{
    /// <summary>
    /// Validates the given command and returns a list of validation errors.
    /// Returns an empty list if the command is valid.
    /// </summary>
    public IReadOnlyList<string> Validate(ProcessPaymentCommand command)
    {
        var errors = new List<string>();

        if (command.UserId == Guid.Empty)
        {
            errors.Add("UserId is required.");
        }

        if (command.Amount <= 0)
        {
            errors.Add("Amount must be greater than zero.");
        }

        if (command.Amount > 1_000_000m)
        {
            errors.Add("Amount exceeds maximum allowed value of 1,000,000.");
        }

        if (string.IsNullOrWhiteSpace(command.Currency))
        {
            errors.Add("Currency is required.");
        }
        else if (command.Currency.Length != 3)
        {
            errors.Add("Currency must be a 3-letter ISO 4217 code.");
        }

        if (string.IsNullOrWhiteSpace(command.PaymentMethodToken))
        {
            errors.Add("PaymentMethodToken is required.");
        }

        if (command.IdempotencyKey is not null && command.IdempotencyKey.Length > 64)
        {
            errors.Add("IdempotencyKey must be 64 characters or fewer.");
        }

        return errors.AsReadOnly();
    }
}
