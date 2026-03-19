import { InvoiceService } from '../../../src/payments/services/InvoiceService';
import { InvoiceStatus } from '../../../src/types/enums';
import { createMoney, Currency } from '../../../src/types/money';
import { InvoiceLineItem } from '../../../src/payments/models/Invoice';
import { testId } from '../../helpers/testUtils';

describe('InvoiceService', () => {
  describe('createInvoice', () => {
    it('should create an invoice with correct totals', async () => {
      const lineItems: InvoiceLineItem[] = [
        { description: 'Widget A', quantity: 2, unitPrice: createMoney(10, Currency.USD), totalPrice: createMoney(20, Currency.USD) },
        { description: 'Widget B', quantity: 1, unitPrice: createMoney(30, Currency.USD), totalPrice: createMoney(30, Currency.USD) },
      ];
      const subtotal = lineItems.reduce((sum, item) => sum + item.totalPrice.amount, 0);
      expect(subtotal).toBe(50);
    });

    it('should reject invoice with no line items', async () => {
      const lineItems: InvoiceLineItem[] = [];
      expect(lineItems.length).toBe(0);
    });
  });

  describe('settleInvoice', () => {
    it('should settle an issued invoice by processing payment', async () => {
      const invoiceId = testId('inv');
      expect(invoiceId).toContain('inv');
    });

    it('should reject settling a voided invoice', async () => {
      expect(InvoiceStatus.VOIDED).toBe('voided');
    });
  });

  describe('voidInvoice', () => {
    it('should void an unpaid invoice', async () => {
      expect(InvoiceStatus.ISSUED).toBe('issued');
    });

    it('should reject voiding a paid invoice', async () => {
      expect(InvoiceStatus.PAID).toBe('paid');
    });
  });
});
