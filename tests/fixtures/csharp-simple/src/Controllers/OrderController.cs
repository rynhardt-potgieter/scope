using CSharpSimple.Payments;

namespace CSharpSimple.Controllers
{
    public class OrderController
    {
        private readonly IPaymentService _paymentService;

        public OrderController(IPaymentService paymentService)
        {
            _paymentService = paymentService;
        }

        public async Task Checkout(decimal amount, string userId)
        {
            await _paymentService.ProcessPayment(amount, userId);
        }
    }
}
