import { Refund } from '../models/Refund';

/** Refund response DTO */
export interface RefundDto {
  id: string;
  paymentId: string;
  userId: string;
  amount: number;
  currency: string;
  reason: string;
  status: string;
  notes: string;
  processedAt: string | null;
  createdAt: string;
}

/** Maps Refund entities to DTOs */
export class RefundMapper {
  /** Convert a Refund entity to a response DTO */
  static toDto(refund: Refund): RefundDto {
    return {
      id: refund.id,
      paymentId: refund.paymentId,
      userId: refund.userId,
      amount: refund.amount.amount,
      currency: refund.amount.currency,
      reason: refund.reason,
      status: refund.status,
      notes: refund.notes,
      processedAt: refund.processedAt?.toISOString() ?? null,
      createdAt: refund.createdAt.toISOString(),
    };
  }

  /** Convert a list of Refunds to DTOs */
  static toDtoList(refunds: Refund[]): RefundDto[] {
    return refunds.map(RefundMapper.toDto);
  }
}
