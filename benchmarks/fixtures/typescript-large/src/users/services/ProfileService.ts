import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';
import { ProfileRepository } from '../repositories/ProfileRepository';
import { UserProfile, SocialLinks } from '../models/UserProfile';
import { EntityId } from '../../types/common';
import { NotFoundError, ValidationError } from '../../types/errors';
import { MAX_UPLOAD_SIZE_BYTES, ALLOWED_IMAGE_TYPES } from '../../types/constants';

/** Service for managing user profiles */
export class ProfileService {
  private profileRepo: ProfileRepository;
  private cache: CacheService;
  private logger: Logger;

  constructor(profileRepo: ProfileRepository, cache: CacheService) {
    this.profileRepo = profileRepo;
    this.cache = cache;
    this.logger = new Logger('ProfileService');
  }

  /** Get a user's profile */
  async getProfile(userId: EntityId): Promise<UserProfile> {
    const cached = this.cache.get<UserProfile>(`profile:${userId}`);
    if (cached) return cached;

    const profile = await this.profileRepo.findByUserId(userId);
    if (!profile) {
      throw new NotFoundError('UserProfile', userId);
    }
    this.cache.set(`profile:${userId}`, profile);
    return profile;
  }

  /** Update a user's profile fields */
  async updateProfile(
    userId: EntityId,
    data: { displayName?: string; bio?: string; timezone?: string; socialLinks?: Partial<SocialLinks> },
  ): Promise<UserProfile> {
    const existing = await this.profileRepo.findByUserId(userId);
    if (!existing) {
      throw new NotFoundError('UserProfile', userId);
    }

    const updated: UserProfile = {
      ...existing,
      displayName: data.displayName ?? existing.displayName,
      bio: data.bio ?? existing.bio,
      timezone: data.timezone ?? existing.timezone,
      socialLinks: { ...existing.socialLinks, ...data.socialLinks },
      updatedAt: new Date(),
    };

    await this.profileRepo.save(updated);
    this.cache.invalidate(`profile:${userId}`);
    this.logger.info('Profile updated', { userId });
    return updated;
  }

  /** Upload a new avatar image */
  async uploadAvatar(userId: EntityId, fileSize: number, mimeType: string, url: string): Promise<string> {
    if (fileSize > MAX_UPLOAD_SIZE_BYTES) {
      throw new ValidationError('File too large', { fileSize: `Maximum ${MAX_UPLOAD_SIZE_BYTES} bytes` });
    }
    if (!ALLOWED_IMAGE_TYPES.includes(mimeType)) {
      throw new ValidationError('Unsupported image type', { mimeType: `Must be one of: ${ALLOWED_IMAGE_TYPES.join(', ')}` });
    }

    const existing = await this.profileRepo.findByUserId(userId);
    if (!existing) {
      throw new NotFoundError('UserProfile', userId);
    }

    await this.profileRepo.save({ ...existing, avatarUrl: url, updatedAt: new Date() });
    this.cache.invalidate(`profile:${userId}`);
    this.logger.info('Avatar uploaded', { userId, mimeType });
    return url;
  }
}
