import { Logger } from '../../shared/utils/Logger';
import { UserService } from '../../users/services/UserService';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { UserDetailResponse, UpdateUserRequest } from '../../users/dtos/UserDtos';
import { createPageRequest } from '../../types/pagination';

/** Controller for user management endpoints */
export class UserController {
  private userService: UserService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(userService: UserService, authGuard: AuthGuard) {
    this.userService = userService;
    this.authGuard = authGuard;
    this.logger = new Logger('UserController');
  }

  /** GET /users/:id */
  async getUser(authHeader: string, userId: string): Promise<ApiResponse<UserDetailResponse>> {
    this.authGuard.guard(authHeader);
    const user = await this.userService.getUser(userId);
    return {
      success: true,
      data: {
        id: user.id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        role: user.role,
        isActive: user.isActive,
        createdAt: user.createdAt.toISOString(),
      },
      message: 'User retrieved',
      timestamp: new Date(),
    };
  }

  /** PUT /users/:id */
  async updateUser(authHeader: string, userId: string, request: UpdateUserRequest): Promise<ApiResponse<UserDetailResponse>> {
    const auth = this.authGuard.guard(authHeader);
    const updated = await this.userService.updateUser(userId, request);
    this.logger.info('User updated via API', { userId, updatedBy: auth.sub });
    return {
      success: true,
      data: {
        id: updated.id,
        email: updated.email,
        firstName: updated.firstName,
        lastName: updated.lastName,
        role: updated.role,
        isActive: updated.isActive,
        createdAt: updated.createdAt.toISOString(),
      },
      message: 'User updated',
      timestamp: new Date(),
    };
  }

  /** DELETE /users/:id */
  async deleteUser(authHeader: string, userId: string): Promise<ApiResponse<{ deleted: boolean }>> {
    const auth = this.authGuard.guard(authHeader);
    await this.userService.deleteUser(userId);
    this.logger.info('User deleted via API', { userId, deletedBy: auth.sub });
    return {
      success: true,
      data: { deleted: true },
      message: 'User deleted',
      timestamp: new Date(),
    };
  }
}
