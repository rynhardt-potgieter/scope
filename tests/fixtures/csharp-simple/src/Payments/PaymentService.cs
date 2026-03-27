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
            this.ValidateAmount(amount);
            return true;
        }

        public async Task<bool> RefundPayment(string transactionId)
        {
            _logger.Info("Refunding");
            return true;
        }

        private bool ValidateAmount(decimal amount)
        {
            base.OnValidating(amount);
            return amount > 0;
        }

        public string DescribeStatus(PaymentStatus status)
        {
            switch (status)
            {
                case PaymentStatus.Pending:
                    return "Pending";
                case PaymentStatus.Completed:
                    return "Done";
                case PaymentStatus.Failed:
                    return "Error";
                default:
                    return "Unknown";
            }
        }
    }
}
