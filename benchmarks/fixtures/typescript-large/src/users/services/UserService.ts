import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';
import { UserRepository } from '../repositories/UserRepository';
import { NotificationService } from '../../notifications/services/NotificationService';
import { UserProfile } from '../models/UserProfile';
import { EntityId } from '../../types/common';
import { PageRequest, PageResponse, buildPageResponse } from '../../types/pagination';
import { NotFoundError } from '../../types/errors';
import { USER_CACHE_TTL_SECONDS } from '../../types/constants';

/** User data returned by the service (no sensitive fields) */
export interface UserInfo {
  id: EntityId;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  isActive: boolean;
  createdAt: Date;
}

/** Core user management service */
export class UserService {
  private userRepo: UserRepository;
  private cache: CacheService;
  private notificationService: NotificationService;
  private logger: Logger;

  constructor(
    userRepo: UserRepository,
    cache: CacheService,
    notificationService: NotificationService,
  ) {
    this.userRepo = userRepo;
    this.cache = cache;
    this.notificationService = notificationService;
    this.logger = new Logger('UserService');
  }

  /** Get a user by ID */
  async getUser(userId: EntityId): Promise<UserInfo> {
    const cached = this.cache.get<UserInfo>(`user:${userId}`);
    if (cached) return cached;

    const user = await this.userRepo.findById(userId);
    if (!user) {
      throw new NotFoundError('User', userId);
    }

    const info: UserInfo = {
      id: user.id,
      email: user.email,
      firstName: user.firstName,
      lastName: user.lastName,
      role: user.role,
      isActive: user.isActive,
      createdAt: user.createdAt,
    };
    this.cache.set(`user:${userId}`, info, USER_CACHE_TTL_SECONDS * 1000);
    return info;
  }

  /** Update a user's basic information */
  async updateUser(userId: EntityId, data: { firstName?: string; lastName?: string }): Promise<UserInfo> {
    const user = await this.userRepo.findById(userId);
    if (!user) {
      throw new NotFoundError('User', userId);
    }

    const updated = await this.userRepo.update(userId, data);
    this.cache.invalidate(`user:${userId}`);
    this.logger.info('User updated', { userId });
    return {
      id: updated!.id,
      email: updated!.email,
      firstName: updated!.firstName,
      lastName: updated!.lastName,
      role: updated!.role,
      isActive: updated!.isActive,
      createdAt: updated!.createdAt,
    };
  }

  /** Soft-delete a user account */
  async deleteUser(userId: EntityId): Promise<void> {
    const deleted = await this.userRepo.delete(userId);
    if (!deleted) {
      throw new NotFoundError('User', userId);
    }
    this.cache.invalidate(`user:${userId}`);
    this.logger.info('User deleted', { userId });

    await this.notificationService.send({
      userId,
      channel: 'email',
      subject: 'Account Deleted',
      body: 'Your account has been deleted. We are sorry to see you go.',
    });
  }

  /** Search users with pagination */
  async searchUsers(query: string, pageRequest: PageRequest): Promise<{ users: UserInfo[]; pagination: PageResponse }> {
    const { users, total } = await this.userRepo.search(query, pageRequest);
    const pagination = buildPageResponse(total, pageRequest);
    this.logger.debug('User search', { query, total });
    return {
      users: users.map((u) => ({
        id: u.id,
        email: u.email,
        firstName: u.firstName,
        lastName: u.lastName,
        role: u.role,
        isActive: u.isActive,
        createdAt: u.createdAt,
      })),
      pagination,
    };
  }
}
