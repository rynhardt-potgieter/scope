import { Logger } from '../../shared/utils/Logger';
import { PaymentRepository } from '../repositories/PaymentRepository';
import { Money, createMoney, Currency } from '../../types/money';
import { PaymentStatus } from '../../types/enums';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Revenue summary for a period */
export interface RevenueSummary {
  totalRevenue: Money;
  totalPayments: number;
  successfulPayments: number;
  failedPayments: number;
  averageAmount: Money;
  successRate: number;
}

/** Analytics service for payment data */
export class PaymentAnalyticsService {
  private paymentRepo: PaymentRepository;
  private logger: Logger;

  constructor(paymentRepo: PaymentRepository) {
    this.paymentRepo = paymentRepo;
    this.logger = new Logger('PaymentAnalyticsService');
  }

  /** Get revenue summary for a date range */
  async getRevenueSummary(start: Date, end: Date): Promise<RevenueSummary> {
    const payments = await this.paymentRepo.findByDateRange(start, end);
    const successful = payments.filter((p) => p.status === PaymentStatus.COMPLETED);
    const failed = payments.filter((p) => p.status === PaymentStatus.FAILED);

    const totalRevenue = successful.reduce((sum, p) => sum + p.amount.amount, 0);
    const avgAmount = successful.length > 0 ? totalRevenue / successful.length : 0;
    const currency = successful[0]?.amount.currency ?? Currency.USD;

    this.logger.info('Revenue summary calculated', {
      period: `${DateUtils.formatDate(start)} to ${DateUtils.formatDate(end)}`,
      totalRevenue,
      totalPayments: payments.length,
    });

    return {
      totalRevenue: createMoney(totalRevenue, currency),
      totalPayments: payments.length,
      successfulPayments: successful.length,
      failedPayments: failed.length,
      averageAmount: createMoney(avgAmount, currency),
      successRate: payments.length > 0 ? successful.length / payments.length : 0,
    };
  }

  /** Get daily revenue for the past N days */
  async getDailyRevenue(days: number): Promise<{ date: string; revenue: number }[]> {
    const results: { date: string; revenue: number }[] = [];
    for (let i = days - 1; i >= 0; i--) {
      const date = DateUtils.addDays(new Date(), -i);
      const start = DateUtils.startOfDay(date);
      const end = DateUtils.endOfDay(date);
      const payments = await this.paymentRepo.findByDateRange(start, end);
      const revenue = payments
        .filter((p) => p.status === PaymentStatus.COMPLETED)
        .reduce((sum, p) => sum + p.amount.amount, 0);
      results.push({ date: DateUtils.formatDate(start), revenue });
    }
    return results;
  }
}
