package payments

// PaymentResult represents the outcome of a payment operation.
type PaymentResult struct {
	ID       string
	Amount   float64
	Fee      float64
	Currency string
	Status   string
}

// CardDetails holds credit card information.
type CardDetails struct {
	Number   string
	ExpMonth int
	ExpYear  int
	CVC      string
}

// Processor is the interface for payment processing backends.
type Processor interface {
	Charge(card CardDetails, amount float64) (*PaymentResult, error)
	Refund(paymentID string) error
}

// Currency is a type alias for string.
type Currency string
