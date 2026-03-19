import { Logger } from '../../shared/utils/Logger';
import { NotificationService } from '../../notifications/services/NotificationService';
import { NotificationRepository } from '../../notifications/repositories/NotificationRepository';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { NotificationDto } from '../../notifications/dtos/NotificationDtos';

/** Controller for notification endpoints */
export class NotificationController {
  private notificationService: NotificationService;
  private notificationRepo: NotificationRepository;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(
    notificationService: NotificationService,
    notificationRepo: NotificationRepository,
    authGuard: AuthGuard,
  ) {
    this.notificationService = notificationService;
    this.notificationRepo = notificationRepo;
    this.authGuard = authGuard;
    this.logger = new Logger('NotificationController');
  }

  /** GET /notifications */
  async getNotifications(authHeader: string): Promise<ApiResponse<NotificationDto[]>> {
    const user = this.authGuard.guard(authHeader);
    const notifications = await this.notificationService.getHistory(user.sub);

    const dtos: NotificationDto[] = notifications.map((n) => ({
      id: n.id,
      channel: n.channel,
      subject: n.subject,
      body: n.body,
      status: n.status,
      sentAt: n.sentAt?.toISOString() ?? null,
      createdAt: n.createdAt.toISOString(),
    }));

    return {
      success: true,
      data: dtos,
      message: `Found ${dtos.length} notifications`,
      timestamp: new Date(),
    };
  }

  /** PUT /notifications/:id/read */
  async markRead(authHeader: string, notificationId: string): Promise<ApiResponse<{ read: boolean }>> {
    this.authGuard.guard(authHeader);
    await this.notificationRepo.markRead(notificationId);
    return {
      success: true,
      data: { read: true },
      message: 'Notification marked as read',
      timestamp: new Date(),
    };
  }
}
