import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { UserProfile } from '../models/UserProfile';
import { EntityId } from '../../types/common';

/** Repository for user profile persistence */
export class ProfileRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('ProfileRepository');
  }

  /** Find a profile by user ID */
  async findByUserId(userId: EntityId): Promise<UserProfile | null> {
    const result = await this.db.query<UserProfile>(
      'SELECT * FROM user_profiles WHERE user_id = $1',
      [userId],
    );
    return result.rows[0] ?? null;
  }

  /** Save or update a user profile */
  async save(profile: UserProfile): Promise<UserProfile> {
    await this.db.execute(
      `INSERT INTO user_profiles (id, user_id, display_name, bio, avatar_url, timezone, locale, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
       ON CONFLICT (user_id) DO UPDATE SET display_name = $3, bio = $4, avatar_url = $5, timezone = $6, updated_at = $9`,
      [profile.id, profile.userId, profile.displayName, profile.bio, profile.avatarUrl, profile.timezone, profile.locale, profile.createdAt, profile.updatedAt],
    );
    this.logger.debug('Profile saved', { userId: profile.userId });
    return profile;
  }
}
