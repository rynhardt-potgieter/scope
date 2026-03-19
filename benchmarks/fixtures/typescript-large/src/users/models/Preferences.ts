import { BaseEntity, EntityId } from '../../types/common';
import { NotificationChannel } from '../../types/enums';

/** User notification and display preferences */
export interface Preferences extends BaseEntity {
  userId: EntityId;
  emailNotifications: boolean;
  pushNotifications: boolean;
  smsNotifications: boolean;
  preferredChannel: NotificationChannel;
  theme: Theme;
  language: string;
  itemsPerPage: number;
}

/** UI theme preference */
export enum Theme {
  LIGHT = 'light',
  DARK = 'dark',
  SYSTEM = 'system',
}
