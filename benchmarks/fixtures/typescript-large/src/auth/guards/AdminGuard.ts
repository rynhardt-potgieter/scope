import { AuthGuard } from './AuthGuard';
import { RoleGuard } from '../middleware/RoleGuard';
import { JwtPayload } from '../services/TokenService';
import { UserRole } from '../../types/enums';
import { Logger } from '../../shared/utils/Logger';

/** Guard that requires both authentication and admin role */
export class AdminGuard {
  private authGuard: AuthGuard;
  private roleGuard: RoleGuard;
  private logger: Logger;

  constructor(authGuard: AuthGuard, roleGuard: RoleGuard) {
    this.authGuard = authGuard;
    this.roleGuard = roleGuard;
    this.logger = new Logger('AdminGuard');
  }

  /** Verify the request is from an authenticated admin user */
  guard(authHeader: string | undefined): JwtPayload {
    const payload = this.authGuard.guard(authHeader);
    this.roleGuard.requireRole(payload.role, [UserRole.ADMIN]);
    this.logger.debug('Admin access granted', { userId: payload.sub });
    return payload;
  }
}
