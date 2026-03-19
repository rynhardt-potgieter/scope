import { ValidationError } from '../../types/errors';
import { Money, Currency } from '../../types/money';

/** Validates payment-related input */
export class PaymentValidator {
  /** Validate that a payment amount is positive and within limits */
  validateAmount(money: Money): void {
    if (money.amount <= 0) {
      throw new ValidationError('Payment amount must be positive', { amount: 'Must be greater than zero' });
    }
    if (money.amount > 1_000_000) {
      throw new ValidationError('Payment amount exceeds maximum limit', { amount: 'Maximum is 1,000,000' });
    }
  }

  /** Validate that the currency is supported */
  validateCurrency(currency: Currency): void {
    const supported = Object.values(Currency);
    if (!supported.includes(currency)) {
      throw new ValidationError(`Unsupported currency: ${currency}`, { currency: `Must be one of: ${supported.join(', ')}` });
    }
  }

  /** Validate a card token format */
  validateCard(cardToken: string): void {
    if (!cardToken || !cardToken.startsWith('tok_')) {
      throw new ValidationError('Invalid card token', { cardToken: 'Must start with tok_' });
    }
    if (cardToken.length < 8) {
      throw new ValidationError('Card token too short', { cardToken: 'Minimum 8 characters' });
    }
  }
}
