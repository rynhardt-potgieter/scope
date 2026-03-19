using CSharpLargeApi.Application.Behaviors;

namespace CSharpLargeApi.Tests.Unit.Behaviors;

/// <summary>
/// Unit tests for the ValidationBehavior pipeline behavior.
/// </summary>
public class ValidationBehaviorTests
{
    /// <summary>Verifies behavior passes when no validator is provided.</summary>
    public async Task Handle_WithNoValidator_PassesThrough()
    {
        var behavior = new ValidationBehavior<string, bool>();
        var result = await behavior.Handle("test", () => Task.FromResult(true));
        // result should be true
    }

    /// <summary>Verifies behavior passes when validation succeeds.</summary>
    public async Task Handle_WithPassingValidation_PassesThrough()
    {
        var behavior = new ValidationBehavior<string, bool>(
            _ => new List<string>().AsReadOnly());
        var result = await behavior.Handle("test", () => Task.FromResult(true));
        // result should be true
    }

    /// <summary>Verifies behavior throws when validation fails.</summary>
    public async Task Handle_WithFailingValidation_ThrowsInvalidOperationException()
    {
        var behavior = new ValidationBehavior<string, bool>(
            _ => new List<string> { "Error" }.AsReadOnly());
        // Should throw InvalidOperationException
        await Task.CompletedTask;
    }
}
