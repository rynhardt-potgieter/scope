import { Logger } from '../../shared/utils/Logger';
import { PaymentAnalyticsService, RevenueSummary } from '../../payments/services/PaymentAnalyticsService';
import { AdminGuard } from '../../auth/guards/AdminGuard';
import { ApiResponse } from '../../types/common';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Controller for analytics endpoints (admin-only) */
export class AnalyticsController {
  private analyticsService: PaymentAnalyticsService;
  private adminGuard: AdminGuard;
  private logger: Logger;

  constructor(analyticsService: PaymentAnalyticsService, adminGuard: AdminGuard) {
    this.analyticsService = analyticsService;
    this.adminGuard = adminGuard;
    this.logger = new Logger('AnalyticsController');
  }

  /** GET /analytics/revenue?start=...&end=... */
  async getRevenueSummary(authHeader: string, startDate: string, endDate: string): Promise<ApiResponse<RevenueSummary>> {
    this.adminGuard.guard(authHeader);
    const start = DateUtils.parseDate(startDate);
    const end = DateUtils.parseDate(endDate);

    const summary = await this.analyticsService.getRevenueSummary(start, end);
    this.logger.info('Revenue summary requested', { startDate, endDate });

    return {
      success: true,
      data: summary,
      message: 'Revenue summary retrieved',
      timestamp: new Date(),
    };
  }

  /** GET /analytics/daily-revenue?days=30 */
  async getDailyRevenue(authHeader: string, days: number = 30): Promise<ApiResponse<{ date: string; revenue: number }[]>> {
    this.adminGuard.guard(authHeader);
    const data = await this.analyticsService.getDailyRevenue(days);
    return {
      success: true,
      data,
      message: `Daily revenue for ${days} days`,
      timestamp: new Date(),
    };
  }
}
