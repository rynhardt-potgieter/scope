import { BaseEntity } from '../../types/common';
import { NotificationChannel } from '../../types/enums';

/** Notification template for generating messages */
export interface Template extends BaseEntity {
  name: string;
  channel: NotificationChannel;
  subject: string;
  bodyTemplate: string;
  variables: string[];
  isActive: boolean;
}

/** Rendered template result */
export interface RenderedTemplate {
  subject: string;
  body: string;
}
