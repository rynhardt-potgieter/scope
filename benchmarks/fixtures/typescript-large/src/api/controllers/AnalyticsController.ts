import { Logger } from '../../shared/utils/Logger';
import { AdminGuard } from '../../auth/guards/AdminGuard';
import { ApiResponse } from '../../types/common';

/** Controller for analytics endpoints (admin-only) */
export class AnalyticsController {
  private adminGuard: AdminGuard;
  private logger: Logger;

  constructor(adminGuard: AdminGuard) {
    this.adminGuard = adminGuard;
    this.logger = new Logger('AnalyticsController');
  }

  /** GET /analytics/revenue?start=...&end=... */
  async getRevenueSummary(authHeader: string, startDate: string, endDate: string): Promise<ApiResponse<never>> {
    this.adminGuard.guard(authHeader);
    throw new Error('PaymentAnalyticsService not implemented — analytics endpoints are not yet available');
  }

  /** GET /analytics/daily-revenue?days=30 */
  async getDailyRevenue(authHeader: string, days: number = 30): Promise<ApiResponse<never>> {
    this.adminGuard.guard(authHeader);
    throw new Error('PaymentAnalyticsService not implemented — analytics endpoints are not yet available');
  }
}
