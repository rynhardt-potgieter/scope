import { Payment } from '../models/Payment';

/** Payment response DTO */
export interface PaymentDto {
  id: string;
  userId: string;
  amount: number;
  currency: string;
  status: string;
  processor: string;
  description: string;
  completedAt: string | null;
  createdAt: string;
}

/** Maps Payment entities to DTOs */
export class PaymentMapper {
  /** Convert a Payment entity to a response DTO */
  static toDto(payment: Payment): PaymentDto {
    return {
      id: payment.id,
      userId: payment.userId,
      amount: payment.amount.amount,
      currency: payment.amount.currency,
      status: payment.status,
      processor: payment.processor,
      description: payment.description,
      completedAt: payment.completedAt?.toISOString() ?? null,
      createdAt: payment.createdAt.toISOString(),
    };
  }

  /** Convert a list of Payment entities to DTOs */
  static toDtoList(payments: Payment[]): PaymentDto[] {
    return payments.map(PaymentMapper.toDto);
  }
}
