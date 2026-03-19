import { Invoice } from '../models/Invoice';

/** Invoice response DTO */
export interface InvoiceDto {
  id: string;
  userId: string;
  invoiceNumber: string;
  status: string;
  subtotal: number;
  tax: number;
  total: number;
  currency: string;
  dueDate: string;
  paidAt: string | null;
  createdAt: string;
}

/** Maps Invoice entities to DTOs */
export class InvoiceMapper {
  /** Convert an Invoice entity to a response DTO */
  static toDto(invoice: Invoice): InvoiceDto {
    return {
      id: invoice.id,
      userId: invoice.userId,
      invoiceNumber: invoice.invoiceNumber,
      status: invoice.status,
      subtotal: invoice.subtotal.amount,
      tax: invoice.tax.amount,
      total: invoice.total.amount,
      currency: invoice.total.currency,
      dueDate: invoice.dueDate.toISOString(),
      paidAt: invoice.paidAt?.toISOString() ?? null,
      createdAt: invoice.createdAt.toISOString(),
    };
  }

  /** Convert a list of Invoice entities to DTOs */
  static toDtoList(invoices: Invoice[]): InvoiceDto[] {
    return invoices.map(InvoiceMapper.toDto);
  }
}
