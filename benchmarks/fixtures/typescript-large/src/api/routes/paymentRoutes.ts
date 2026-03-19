/** Route definitions for payment endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns payment route definitions */
export function paymentRoutes(): RouteDefinition[] {
  return [
    { method: 'POST', path: '/payments', handler: 'PaymentController.createPayment' },
    { method: 'GET', path: '/payments/:id', handler: 'PaymentController.getPayment' },
    { method: 'GET', path: '/payments', handler: 'PaymentController.listPayments' },
    { method: 'POST', path: '/orders/checkout', handler: 'OrderController.checkout' },
    { method: 'POST', path: '/orders/:id/retry-payment', handler: 'OrderController.retryPayment' },
    { method: 'POST', path: '/orders/:id/cancel', handler: 'OrderController.cancelOrder' },
    { method: 'POST', path: '/refunds', handler: 'RefundController.processRefund' },
    { method: 'POST', path: '/refunds/partial', handler: 'RefundController.processPartialRefund' },
  ];
}
