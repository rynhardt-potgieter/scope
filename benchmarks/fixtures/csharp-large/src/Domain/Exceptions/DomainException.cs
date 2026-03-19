namespace CSharpLargeApi.Domain.Exceptions;

/// <summary>
/// Base exception for all domain-level errors.
/// Indicates a violation of domain rules or invariants.
/// </summary>
public class DomainException : Exception
{
    /// <summary>
    /// Gets the error code associated with this domain exception.
    /// </summary>
    public string ErrorCode { get; }

    /// <summary>
    /// Creates a new DomainException with a message and error code.
    /// </summary>
    public DomainException(string message, string errorCode = "DOMAIN_ERROR")
        : base(message)
    {
        ErrorCode = errorCode;
    }

    /// <summary>
    /// Creates a new DomainException with a message, error code, and inner exception.
    /// </summary>
    public DomainException(string message, string errorCode, Exception innerException)
        : base(message, innerException)
    {
        ErrorCode = errorCode;
    }
}
