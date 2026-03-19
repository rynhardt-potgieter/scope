import { Logger } from '../../shared/utils/Logger';
import { UserRole } from '../../types/enums';
import { ForbiddenError } from '../../types/errors';

/** Middleware that enforces role-based access control */
export class RoleGuard {
  private logger: Logger;

  constructor() {
    this.logger = new Logger('RoleGuard');
  }

  /** Check that the user's role is in the allowed list */
  requireRole(userRole: string, allowedRoles: UserRole[]): void {
    if (!allowedRoles.includes(userRole as UserRole)) {
      this.logger.warn('Role check failed', { userRole, allowedRoles });
      throw new ForbiddenError(`Role ${userRole} is not authorized for this action`);
    }
    this.logger.debug('Role check passed', { userRole });
  }

  /** Check that the user is at least at the minimum role level */
  requireMinimumRole(userRole: string, minimumRole: UserRole): void {
    const hierarchy: UserRole[] = [UserRole.VIEWER, UserRole.EDITOR, UserRole.MANAGER, UserRole.ADMIN];
    const userLevel = hierarchy.indexOf(userRole as UserRole);
    const requiredLevel = hierarchy.indexOf(minimumRole);

    if (userLevel < requiredLevel) {
      this.logger.warn('Minimum role check failed', { userRole, minimumRole });
      throw new ForbiddenError(`Requires at least ${minimumRole} role`);
    }
  }
}
