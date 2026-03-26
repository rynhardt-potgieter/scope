package com.example.payments;

/**
 * Interface for payment processing.
 */
public interface IPaymentClient {

    PaymentResult processPayment(String orderId, double amount);
}
