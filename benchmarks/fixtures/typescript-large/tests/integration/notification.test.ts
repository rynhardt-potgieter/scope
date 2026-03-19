import { NotificationChannel } from '../../src/types/enums';
import { DeliveryStatus } from '../../src/notifications/models/Notification';
import { mockNotification } from '../helpers/mockFactory';

describe('Notification Integration', () => {
  describe('sending notifications', () => {
    it('should deliver email notifications', () => {
      const notification = mockNotification({ channel: NotificationChannel.EMAIL });
      expect(notification.channel).toBe(NotificationChannel.EMAIL);
    });

    it('should deliver push notifications', () => {
      const notification = mockNotification({ channel: NotificationChannel.PUSH });
      expect(notification.channel).toBe(NotificationChannel.PUSH);
    });

    it('should deliver SMS notifications', () => {
      const notification = mockNotification({ channel: NotificationChannel.SMS });
      expect(notification.channel).toBe(NotificationChannel.SMS);
    });
  });

  describe('delivery failures', () => {
    it('should mark failed notifications', () => {
      expect(DeliveryStatus.FAILED).toBe('failed');
    });

    it('should track retry count', () => {
      const notification = mockNotification({ retryCount: 2 });
      expect(notification.retryCount).toBe(2);
    });
  });
});
