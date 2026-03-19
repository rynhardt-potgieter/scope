namespace CSharpLargeApi.Domain.Exceptions;

/// <summary>
/// Exception thrown when a business rule is violated.
/// Contains details about which rule was broken and suggested remediation.
/// </summary>
public class BusinessRuleException : DomainException
{
    /// <summary>
    /// Gets the name of the business rule that was violated.
    /// </summary>
    public string RuleName { get; }

    /// <summary>
    /// Gets the entity or aggregate that the rule applies to.
    /// </summary>
    public string? EntityName { get; }

    /// <summary>
    /// Creates a new BusinessRuleException with rule details.
    /// </summary>
    public BusinessRuleException(string ruleName, string message, string? entityName = null)
        : base(message, "BUSINESS_RULE_VIOLATION")
    {
        RuleName = ruleName;
        EntityName = entityName;
    }

    /// <summary>
    /// Creates a formatted message including the rule name.
    /// </summary>
    public string GetDetailedMessage()
    {
        var detail = $"Business rule '{RuleName}' violated: {Message}";
        if (!string.IsNullOrEmpty(EntityName))
        {
            detail += $" (Entity: {EntityName})";
        }
        return detail;
    }
}
