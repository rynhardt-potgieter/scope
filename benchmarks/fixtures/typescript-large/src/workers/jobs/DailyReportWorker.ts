import { Logger } from '../../shared/utils/Logger';
import { PaymentRepository } from '../../payments/repositories/PaymentRepository';
import { NotificationService } from '../../notifications/services/NotificationService';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Daily report data */
interface DailyReport {
  date: string;
  totalPayments: number;
  totalRevenue: number;
  failedPayments: number;
  newSubscriptions: number;
}

/** Background worker that generates and emails daily reports */
export class DailyReportWorker {
  private paymentRepo: PaymentRepository;
  private notificationService: NotificationService;
  private logger: Logger;

  constructor(paymentRepo: PaymentRepository, notificationService: NotificationService) {
    this.paymentRepo = paymentRepo;
    this.notificationService = notificationService;
    this.logger = new Logger('DailyReportWorker');
  }

  /** Generate the daily report for the previous day */
  async generateDailyReport(): Promise<DailyReport> {
    const yesterday = DateUtils.addDays(new Date(), -1);
    const start = DateUtils.startOfDay(yesterday);
    const end = DateUtils.endOfDay(yesterday);

    this.logger.info('Generating daily report', { date: DateUtils.formatDate(start) });

    const payments = await this.paymentRepo.findByDateRange(start, end);
    const completed = payments.filter((p) => p.status === 'completed');
    const failed = payments.filter((p) => p.status === 'failed');

    const report: DailyReport = {
      date: DateUtils.formatDate(start),
      totalPayments: payments.length,
      totalRevenue: completed.reduce((sum, p) => sum + p.amount.amount, 0),
      failedPayments: failed.length,
      newSubscriptions: 0,
    };

    this.logger.info('Daily report generated', {
      date: report.date,
      totalPayments: report.totalPayments,
      totalRevenue: report.totalRevenue,
    });

    return report;
  }
}
