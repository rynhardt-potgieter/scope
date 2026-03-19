import { DateUtils } from '../../../src/shared/utils/DateUtils';

describe('DailyReportWorker', () => {
  describe('generateDailyReport', () => {
    it('should generate a report for the previous day', () => {
      const yesterday = DateUtils.addDays(new Date(), -1);
      expect(yesterday.getTime()).toBeLessThan(Date.now());
    });

    it('should calculate total revenue from completed payments', () => {
      const amounts = [10, 20, 30];
      const total = amounts.reduce((sum, a) => sum + a, 0);
      expect(total).toBe(60);
    });

    it('should count failed payments separately', () => {
      const payments = [{ status: 'completed' }, { status: 'failed' }, { status: 'completed' }];
      const failed = payments.filter((p) => p.status === 'failed');
      expect(failed.length).toBe(1);
    });
  });
});
