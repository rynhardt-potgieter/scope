package com.example.controllers;

import com.example.payments.PaymentService;
import com.example.payments.PaymentResult;
import com.example.utils.Logger;

/**
 * Controller for order operations.
 */
public class OrderController {

    private final PaymentService paymentService;
    private final Logger logger;

    public OrderController(PaymentService paymentService, Logger logger) {
        this.paymentService = paymentService;
        this.logger = logger;
    }

    public PaymentResult createOrder(String orderId, double amount) {
        logger.info("Creating order: " + orderId);
        PaymentResult result = paymentService.processPayment(orderId, amount);
        return result;
    }

    public void cancelOrder(String orderId) {
        logger.info("Cancelling order: " + orderId);
    }
}
