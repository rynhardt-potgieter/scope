package com.example.payments;

/**
 * Exception thrown when payment processing fails.
 */
public class PaymentException extends Exception {

    public PaymentException(String message) {
        super(message);
    }
}
