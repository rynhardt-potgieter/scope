import { Logger } from '../../shared/utils/Logger';
import { Preferences, Theme } from '../models/Preferences';
import { EntityId } from '../../types/common';
import { NotificationChannel } from '../../types/enums';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { NotFoundError } from '../../types/errors';
import { DEFAULT_PAGE_SIZE } from '../../types/constants';

/** Service for managing user preferences */
export class PreferencesService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('PreferencesService');
  }

  /** Get preferences for a user, creating defaults if none exist */
  async getPreferences(userId: EntityId): Promise<Preferences> {
    const result = await this.db.query<Preferences>('SELECT * FROM preferences WHERE user_id = $1', [userId]);
    if (result.rows.length > 0) {
      return result.rows[0];
    }
    return this.createDefaults(userId);
  }

  /** Update user preferences */
  async updatePreferences(
    userId: EntityId,
    data: Partial<Pick<Preferences, 'emailNotifications' | 'pushNotifications' | 'smsNotifications' | 'preferredChannel' | 'theme' | 'language' | 'itemsPerPage'>>,
  ): Promise<Preferences> {
    const existing = await this.getPreferences(userId);
    const updated: Preferences = {
      ...existing,
      ...data,
      updatedAt: new Date(),
    };

    await this.db.execute(
      'UPDATE preferences SET email_notifications = $1, push_notifications = $2, theme = $3, language = $4, items_per_page = $5, updated_at = $6 WHERE user_id = $7',
      [updated.emailNotifications, updated.pushNotifications, updated.theme, updated.language, updated.itemsPerPage, updated.updatedAt, userId],
    );
    this.logger.info('Preferences updated', { userId });
    return updated;
  }

  private async createDefaults(userId: EntityId): Promise<Preferences> {
    const now = new Date();
    const defaults: Preferences = {
      id: `pref_${Date.now()}`,
      userId,
      emailNotifications: true,
      pushNotifications: true,
      smsNotifications: false,
      preferredChannel: NotificationChannel.EMAIL,
      theme: Theme.SYSTEM,
      language: 'en',
      itemsPerPage: DEFAULT_PAGE_SIZE,
      createdAt: now,
      updatedAt: now,
    };
    await this.db.execute(
      'INSERT INTO preferences (id, user_id, email_notifications, push_notifications, theme, language, items_per_page, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)',
      [defaults.id, userId, defaults.emailNotifications, defaults.pushNotifications, defaults.theme, defaults.language, defaults.itemsPerPage, now],
    );
    this.logger.info('Default preferences created', { userId });
    return defaults;
  }
}
