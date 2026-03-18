// Unit tests for PaymentService
// In a real project these would use xUnit or NUnit with a test runner.
// For the benchmark fixture, compilation is the primary check.

using System.Diagnostics;
using CSharpApi.Payments;
using CSharpApi.Notifications;

namespace CSharpApi.Tests;

/// <summary>
/// Tests for the PaymentService class covering all public methods.
/// </summary>
public static class PaymentServiceTests
{
    private static readonly CardDetails TestCard = new(
        CardNumber: "4111111111111111",
        ExpiryMonth: 12,
        ExpiryYear: 2030,
        Cvv: "123",
        CardholderName: "Test User"
    );

    public static void RunAll()
    {
        TestProcessPaymentSuccess();
        TestRefundPaymentNotFound();
        TestValidateCardRejectsShortNumber();
        TestValidateCardRejectsExpired();
        TestValidateCardRejectsShortCvv();
        TestGetTransactionReturnsNullForMissing();
        TestProcessorChargeRejectsNegativeAmount();
        TestProcessorRefundSuccess();
        TestRefundPaymentNonexistentTransaction();

        Console.WriteLine("All payment tests passed");
    }

    private static void TestProcessPaymentSuccess()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);

        // Call ProcessPayment — caller #7 (test)
        var result = service.ProcessPayment(100.00m, "user_1", TestCard);

        Debug.Assert(result.Status == "success", "Expected success status");
        Debug.Assert(result.Amount == 100.00m, "Expected amount 100.00");
        Debug.Assert(!string.IsNullOrEmpty(result.TransactionId), "Expected non-empty transaction ID");

        // Verify transaction is stored
        var stored = service.GetTransaction(result.TransactionId);
        Debug.Assert(stored != null, "Transaction should be stored");
        Debug.Assert(stored!.Amount == 100.00m, "Stored amount should match");

        // Verify refund works on the stored transaction
        var refund = service.RefundPayment(result.TransactionId);
        Debug.Assert(refund.Status == "success", "Expected refund success");
    }

    private static void TestRefundPaymentNotFound()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);

        var result = service.RefundPayment("nonexistent");
        Debug.Assert(result.Status == "failed", "Expected failed status");
        Debug.Assert(result.ErrorMessage != null, "Expected error message");
    }

    private static void TestValidateCardRejectsShortNumber()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);
        var badCard = TestCard with { CardNumber = "123" };

        try
        {
            service.ValidateCard(badCard);
            Debug.Assert(false, "Should have thrown");
        }
        catch (ArgumentException)
        {
            // Expected
        }
    }

    private static void TestValidateCardRejectsExpired()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);
        var expiredCard = TestCard with { ExpiryYear = 2020 };

        try
        {
            service.ValidateCard(expiredCard);
            Debug.Assert(false, "Should have thrown");
        }
        catch (ArgumentException)
        {
            // Expected
        }
    }

    private static void TestValidateCardRejectsShortCvv()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);
        var badCard = TestCard with { Cvv = "1" };

        try
        {
            service.ValidateCard(badCard);
            Debug.Assert(false, "Should have thrown for bad CVV");
        }
        catch (ArgumentException)
        {
            // Expected
        }
    }

    private static void TestGetTransactionReturnsNullForMissing()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);

        var result = service.GetTransaction("missing");
        Debug.Assert(result == null, "Expected null for missing transaction");
    }

    private static void TestProcessorChargeRejectsNegativeAmount()
    {
        var processor = new PaymentProcessor("test_key");

        var result = processor.Charge(-50.00m, "USD", TestCard);
        Debug.Assert(result.Status == "failed", "Expected failed status for negative amount");
        Debug.Assert(result.ErrorMessage != null, "Expected error message");
    }

    private static void TestProcessorRefundSuccess()
    {
        var processor = new PaymentProcessor("test_key");

        var chargeResult = processor.Charge(200.00m, "USD", TestCard);
        var refundResult = processor.Refund(chargeResult.TransactionId, chargeResult.Amount);
        Debug.Assert(refundResult.Status == "success", "Expected refund success");
        Debug.Assert(refundResult.TransactionId.StartsWith("ref_"), "Expected refund transaction ID prefix");
    }

    private static void TestRefundPaymentNonexistentTransaction()
    {
        var processor = new PaymentProcessor("test_key");
        var notifications = new NotificationService("https://test.email", "+15550000000");
        var service = new PaymentService(processor, notifications);

        var result = service.RefundPayment("txn_does_not_exist");
        Debug.Assert(result.Status == "failed", "Expected failed for nonexistent transaction");
    }
}
