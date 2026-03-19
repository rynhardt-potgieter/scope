import { AuthMiddleware } from '../middleware/AuthMiddleware';
import { JwtPayload } from '../services/TokenService';
import { Logger } from '../../shared/utils/Logger';

/** Route-level authentication guard */
export class AuthGuard {
  private authMiddleware: AuthMiddleware;
  private logger: Logger;

  constructor(authMiddleware: AuthMiddleware) {
    this.authMiddleware = authMiddleware;
    this.logger = new Logger('AuthGuard');
  }

  /** Guard a route — verifies the token and returns the payload */
  guard(authHeader: string | undefined): JwtPayload {
    const payload = this.authMiddleware.authenticate(authHeader);
    this.logger.debug('Route guarded', { userId: payload.sub });
    return payload;
  }

  /** Guard with optional auth — returns null if no header, payload if valid */
  optionalGuard(authHeader: string | undefined): JwtPayload | null {
    if (!authHeader) return null;
    try {
      return this.authMiddleware.authenticate(authHeader);
    } catch {
      return null;
    }
  }
}
