using CSharpSimple.Utils;

namespace CSharpSimple.Payments
{
    public class PaymentService : IPaymentService
    {
        private readonly Logger _logger;

        public PaymentService(Logger logger)
        {
            _logger = logger;
        }

        public async Task<bool> ProcessPayment(decimal amount, string userId)
        {
            _logger.Info("Processing payment");
            return true;
        }

        public async Task<bool> RefundPayment(string transactionId)
        {
            _logger.Info("Refunding");
            return true;
        }

        private bool ValidateAmount(decimal amount)
        {
            return amount > 0;
        }
    }
}
