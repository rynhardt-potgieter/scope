/** Notification response DTO */
export interface NotificationDto {
  id: string;
  channel: string;
  subject: string;
  body: string;
  status: string;
  sentAt: string | null;
  createdAt: string;
}

/** Request to send a notification */
export interface SendNotificationDto {
  userId: string;
  channel: string;
  subject: string;
  body: string;
}

/** Bulk notification request */
export interface BulkNotificationRequest {
  userIds: string[];
  channel: string;
  subject: string;
  body: string;
}

/** Notification preferences update */
export interface NotificationPreferencesDto {
  emailEnabled: boolean;
  pushEnabled: boolean;
  smsEnabled: boolean;
  preferredChannel: string;
}
