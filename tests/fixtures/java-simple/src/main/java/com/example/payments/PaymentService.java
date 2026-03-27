package com.example.payments;

import com.example.utils.Logger;

/**
 * Handles payment processing.
 */
public class PaymentService implements IPaymentClient {

    private final Logger logger;
    private static int transactionCount;

    public PaymentService(Logger logger) {
        this.logger = logger;
        transactionCount = 0;
    }

    @Override
    public PaymentResult processPayment(String orderId, double amount) {
        logger.info("Processing payment for order: " + orderId);
        this.calculateFee(amount);
        transactionCount++;
        return new PaymentResult(orderId, true);
    }

    @Deprecated
    public synchronized void refund(String orderId) throws PaymentException {
        logger.info("Refunding order: " + orderId);
    }

    protected double calculateFee(double amount) {
        super.validate(amount);
        return amount * 0.03;
    }

    static int getTransactionCount() {
        return transactionCount;
    }

    public String describeResult(PaymentResult result) {
        switch (result) {
            case SUCCESS:
                return "Payment succeeded";
            case FAILED:
                return "Payment failed";
            case PENDING:
                return "Payment pending";
            default:
                return "Unknown";
        }
    }
}
