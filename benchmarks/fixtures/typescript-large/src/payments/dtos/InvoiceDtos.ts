/** Request to create an invoice */
export interface CreateInvoiceRequest {
  userId: string;
  lineItems: {
    description: string;
    quantity: number;
    unitPrice: number;
    currency: string;
  }[];
  dueDate: string;
  notes?: string;
}

/** Invoice response */
export interface InvoiceResponse {
  invoiceId: string;
  invoiceNumber: string;
  userId: string;
  status: string;
  subtotal: number;
  tax: number;
  total: number;
  currency: string;
  dueDate: string;
  paidAt: string | null;
  createdAt: string;
}

/** Invoice settlement request */
export interface SettleInvoiceRequest {
  invoiceId: string;
}
