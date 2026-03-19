import { BaseEntity, EntityId } from '../../types/common';
import { InvoiceStatus } from '../../types/enums';
import { Money } from '../../types/money';

/** Line item on an invoice */
export interface InvoiceLineItem {
  description: string;
  quantity: number;
  unitPrice: Money;
  totalPrice: Money;
}

/** An invoice tied to a subscription or one-time purchase */
export interface Invoice extends BaseEntity {
  userId: EntityId;
  subscriptionId: EntityId | null;
  paymentId: EntityId | null;
  invoiceNumber: string;
  status: InvoiceStatus;
  lineItems: InvoiceLineItem[];
  subtotal: Money;
  tax: Money;
  total: Money;
  dueDate: Date;
  paidAt: Date | null;
  notes: string;
}
