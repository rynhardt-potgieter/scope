package payments

import (
	"fmt"

	"github.com/example/payments/utils"
)

// DefaultCurrency is the fallback currency for payments.
const DefaultCurrency = "USD"

// PaymentService handles payment processing.
type PaymentService struct {
	utils.Logger
	currency string
}

// NewPaymentService creates a new PaymentService with the given logger.
func NewPaymentService(logger utils.Logger) *PaymentService {
	return &PaymentService{
		Logger:   logger,
		currency: DefaultCurrency,
	}
}

// ProcessPayment processes a payment for the given card and amount.
func (s *PaymentService) ProcessPayment(cardID string, amount float64) (*PaymentResult, error) {
	if !validateCard(cardID) {
		return nil, fmt.Errorf("invalid card: %s", cardID)
	}

	s.Info(fmt.Sprintf("processing payment of %.2f %s", amount, s.currency))

	fee := calculateFee(amount)

	return &PaymentResult{
		ID:       fmt.Sprintf("pay_%s", cardID),
		Amount:   amount,
		Fee:      fee,
		Currency: s.currency,
		Status:   "completed",
	}, nil
}

// Refund issues a refund for a previous payment.
func (s *PaymentService) Refund(paymentID string) error {
	s.Info(fmt.Sprintf("refunding payment %s", paymentID))
	return nil
}

// SetCurrency changes the default currency (value receiver).
func (s PaymentService) SetCurrency(currency string) PaymentService {
	s.currency = currency
	return s
}

// validateCard checks if a card ID is valid (unexported function).
func validateCard(cardID string) bool {
	return len(cardID) > 0
}

// calculateFee computes the processing fee for a given amount.
func calculateFee(amount float64) float64 {
	return amount * 0.029
}
