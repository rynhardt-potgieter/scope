package main

import (
	"fmt"

	"github.com/example/payments/payments"
	"github.com/example/payments/utils"
)

// MaxConnections is the maximum number of connections allowed.
const MaxConnections = 100

// main is the entry point for the application.
func main() {
	logger := utils.NewLogger("app")
	service := payments.NewPaymentService(logger)

	result, err := service.ProcessPayment("card_123", 99.99)
	if err != nil {
		logger.Error(fmt.Sprintf("payment failed: %v", err))
		return
	}

	logger.Info(fmt.Sprintf("payment succeeded: %s", result.ID))
}
