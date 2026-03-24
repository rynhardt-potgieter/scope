/// Result of a payment operation.
pub enum PaymentResult {
    Success { tx_id: String },
    Failure { reason: String },
}

/// Credit card details.
pub struct CardDetails {
    pub number: Option<String>,
    pub expiry: String,
    pub cvv: String,
}

/// Supported payment methods.
pub enum PaymentMethod {
    CreditCard(CardDetails),
    BankTransfer { account: String },
}

/// A constant representing the maximum payment amount.
pub const MAX_PAYMENT_AMOUNT: f64 = 100_000.0;

/// Type alias for transaction identifiers.
pub type TransactionId = String;
