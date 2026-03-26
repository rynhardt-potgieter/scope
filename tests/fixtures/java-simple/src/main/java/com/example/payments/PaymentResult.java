package com.example.payments;

/**
 * Result of a payment operation.
 */
public enum PaymentResult {
    SUCCESS,
    FAILED,
    PENDING;

    private String orderId;
    private boolean completed;

    PaymentResult() {
        this.orderId = "";
        this.completed = false;
    }

    PaymentResult(String orderId, boolean completed) {
        this.orderId = orderId;
        this.completed = completed;
    }

    public String getOrderId() {
        return orderId;
    }

    public boolean isCompleted() {
        return completed;
    }
}
