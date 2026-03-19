import { TokenService, JwtPayload } from '../services/TokenService';
import { Logger } from '../../shared/utils/Logger';
import { UnauthorizedError } from '../../types/errors';

/** Authenticated request with user context */
export interface AuthenticatedRequest {
  userId: string;
  email: string;
  role: string;
  headers: Record<string, string>;
}

/** Middleware that extracts and verifies JWT tokens from request headers */
export class AuthMiddleware {
  private tokenService: TokenService;
  private logger: Logger;

  constructor(tokenService: TokenService) {
    this.tokenService = tokenService;
    this.logger = new Logger('AuthMiddleware');
  }

  /** Verify the Authorization header and return the decoded payload */
  authenticate(authHeader: string | undefined): JwtPayload {
    if (!authHeader) {
      throw new UnauthorizedError('Missing Authorization header');
    }

    if (!authHeader.startsWith('Bearer ')) {
      throw new UnauthorizedError('Invalid Authorization header format');
    }

    const token = authHeader.slice(7);
    try {
      const payload = this.tokenService.verifyJwt(token);
      this.logger.debug('Request authenticated', { userId: payload.sub });
      return payload;
    } catch (error) {
      this.logger.warn('Authentication failed', { error: String(error) });
      throw new UnauthorizedError('Invalid or expired token');
    }
  }
}
