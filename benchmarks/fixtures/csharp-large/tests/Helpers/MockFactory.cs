using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Helpers;

/// <summary>
/// Factory methods for creating test entities with sensible defaults.
/// Reduces boilerplate in test setup by providing pre-configured objects.
/// </summary>
public static class MockFactory
{
    /// <summary>
    /// Creates a test user with default values.
    /// </summary>
    public static User CreateUser(
        string displayName = "Test User",
        string email = "test@example.com",
        UserRole role = UserRole.User)
    {
        var emailAddress = new EmailAddress(email);
        return User.Create(displayName, emailAddress, "hashed_password_123", role);
    }

    /// <summary>
    /// Creates a test payment with default values.
    /// </summary>
    public static Payment CreatePayment(
        Guid? userId = null,
        decimal amount = 100.00m,
        string currency = "USD")
    {
        var money = new Money(amount, currency);
        return Payment.Create(userId ?? Guid.NewGuid(), money);
    }

    /// <summary>
    /// Creates a test invoice with default values.
    /// </summary>
    public static Invoice CreateInvoice(
        Guid? userId = null,
        string currency = "USD")
    {
        var invoiceNumber = $"INV-TEST-{Guid.NewGuid():N}"[..20];
        return Invoice.Create(
            userId ?? Guid.NewGuid(),
            invoiceNumber,
            DateTime.UtcNow.AddDays(30),
            currency);
    }

    /// <summary>
    /// Creates a test subscription with default values.
    /// </summary>
    public static Subscription CreateSubscription(
        Guid? userId = null,
        string planName = "Pro",
        decimal monthlyPrice = 29.99m,
        string currency = "USD")
    {
        var price = new Money(monthlyPrice, currency);
        var billingPeriod = new DateRange(DateTime.UtcNow, DateTime.UtcNow.AddMonths(1));
        return Subscription.Create(userId ?? Guid.NewGuid(), planName, price, billingPeriod);
    }

    /// <summary>
    /// Creates a test content item with default values.
    /// </summary>
    public static Content CreateContent(
        Guid? authorId = null,
        string title = "Test Article",
        string slug = "test-article")
    {
        return Content.Create(
            authorId ?? Guid.NewGuid(),
            title,
            slug,
            "This is the body of the test article. It contains enough text to be realistic.");
    }

    /// <summary>
    /// Creates a test notification with default values.
    /// </summary>
    public static Notification CreateNotification(
        Guid? recipientId = null,
        NotificationChannel channel = NotificationChannel.Email)
    {
        return Notification.Create(
            recipientId ?? Guid.NewGuid(),
            "Test Subject",
            "Test notification body content.",
            channel);
    }

    /// <summary>
    /// Creates a test Money value object.
    /// </summary>
    public static Money CreateMoney(decimal amount = 100.00m, string currency = "USD")
    {
        return new Money(amount, currency);
    }

    /// <summary>
    /// Creates a test EmailAddress value object.
    /// </summary>
    public static EmailAddress CreateEmail(string email = "test@example.com")
    {
        return new EmailAddress(email);
    }
}
