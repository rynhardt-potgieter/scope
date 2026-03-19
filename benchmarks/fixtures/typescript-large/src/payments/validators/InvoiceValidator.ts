import { ValidationError } from '../../types/errors';
import { InvoiceLineItem } from '../models/Invoice';

/** Validates invoice-related input */
export class InvoiceValidator {
  /** Validate that an invoice has valid line items */
  validateInvoice(lineItems: InvoiceLineItem[], dueDate: Date): void {
    const errors: Record<string, string> = {};

    if (!lineItems || lineItems.length === 0) {
      errors.lineItems = 'At least one line item is required';
    }

    for (let i = 0; i < lineItems.length; i++) {
      const item = lineItems[i];
      if (!item.description || item.description.trim().length === 0) {
        errors[`lineItems[${i}].description`] = 'Description is required';
      }
      if (item.quantity <= 0) {
        errors[`lineItems[${i}].quantity`] = 'Quantity must be positive';
      }
      if (item.unitPrice.amount < 0) {
        errors[`lineItems[${i}].unitPrice`] = 'Unit price cannot be negative';
      }
    }

    if (dueDate < new Date()) {
      errors.dueDate = 'Due date must be in the future';
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Invoice validation failed', errors);
    }
  }
}
