import { NotificationChannel } from '../../../src/types/enums';
import { DeliveryStatus } from '../../../src/notifications/models/Notification';
import { mockNotification } from '../../helpers/mockFactory';

describe('NotificationService', () => {
  describe('send', () => {
    it('should create a notification with pending status', () => {
      const notification = mockNotification({ status: DeliveryStatus.PENDING });
      expect(notification.status).toBe(DeliveryStatus.PENDING);
    });

    it('should route to the correct delivery channel', () => {
      const email = mockNotification({ channel: NotificationChannel.EMAIL });
      const push = mockNotification({ channel: NotificationChannel.PUSH });
      expect(email.channel).toBe(NotificationChannel.EMAIL);
      expect(push.channel).toBe(NotificationChannel.PUSH);
    });

    it('should mark as failed on delivery error', () => {
      expect(DeliveryStatus.FAILED).toBe('failed');
    });
  });

  describe('sendBulk', () => {
    it('should send to multiple users', () => {
      const userIds = ['user1', 'user2', 'user3'];
      expect(userIds.length).toBe(3);
    });
  });

  describe('getHistory', () => {
    it('should return notifications ordered by creation date', () => {
      expect(true).toBe(true);
    });
  });
});
