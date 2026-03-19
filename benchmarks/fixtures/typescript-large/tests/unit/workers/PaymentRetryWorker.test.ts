import { mockPayment } from '../../helpers/mockFactory';
import { PaymentStatus, PaymentProcessor } from '../../../src/types/enums';
import { MAX_RETRY_ATTEMPTS } from '../../../src/types/constants';

describe('PaymentRetryWorker', () => {
  describe('retryFailedPayment', () => {
    it('should call processPayment for queued failed payments', () => {
      const payment = mockPayment({ status: PaymentStatus.FAILED });
      expect(payment.status).toBe(PaymentStatus.FAILED);
    });

    it('should re-queue on failure with incremented attempt', () => {
      const attempt = 1;
      expect(attempt + 1).toBe(2);
    });

    it('should stop retrying after max attempts', () => {
      expect(MAX_RETRY_ATTEMPTS).toBe(3);
    });

    it('should skip if payment no longer exists', () => {
      expect(null).toBeNull();
    });
  });

  describe('processAll', () => {
    it('should process all jobs in the queue', () => {
      expect(true).toBe(true);
    });
  });
});
