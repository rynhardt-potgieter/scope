use crate::payments::types::{CardDetails, PaymentResult};
use crate::utils::logger::Logger;

/// Handles payment processing.
pub struct PaymentService {
    client: Box<dyn PaymentClient>,
    logger: Logger,
}

impl PaymentService {
    /// Create a new payment service.
    pub fn new(client: Box<dyn PaymentClient>, logger: Logger) -> Self {
        Self { client, logger }
    }

    /// Process a payment for a given user.
    pub async fn process_payment(&self, amount: f64, user_id: &str) -> PaymentResult {
        self.logger.info(&format!("Processing payment for {user_id}"));
        self.client.charge(amount, user_id).await
    }

    /// Refund a transaction.
    pub fn refund(&self, tx_id: &str, reason: Option<&str>) -> bool {
        self.logger.info("Processing refund");
        self.client.refund(tx_id, reason)
    }

    fn validate_card(card: &CardDetails) -> bool {
        card.number.is_some()
    }
}

/// Trait for payment processing backends.
pub trait PaymentClient: Send + Sync {
    /// Charge a user.
    async fn charge(&self, amount: f64, user_id: &str) -> PaymentResult;
    /// Refund a transaction.
    fn refund(&self, tx_id: &str, reason: Option<&str>) -> bool;
}

/// A mock payment client for testing.
pub struct MockPaymentClient;

impl PaymentClient for MockPaymentClient {
    async fn charge(&self, _amount: f64, _user_id: &str) -> PaymentResult {
        PaymentResult::Success {
            tx_id: "mock-tx-001".to_string(),
        }
    }

    fn refund(&self, _tx_id: &str, _reason: Option<&str>) -> bool {
        true
    }
}
