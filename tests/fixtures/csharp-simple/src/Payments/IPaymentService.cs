namespace CSharpSimple.Payments
{
    public interface IPaymentService
    {
        Task<bool> ProcessPayment(decimal amount, string userId);
        Task<bool> RefundPayment(string transactionId);
    }
}
