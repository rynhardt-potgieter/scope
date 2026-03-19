import { Logger } from '../../shared/utils/Logger';
import { AuthConfig } from '../../shared/config/AuthConfig';
import { User } from '../models/User';
import { UnauthorizedError } from '../../types/errors';

/** Decoded JWT payload */
export interface JwtPayload {
  sub: string;
  email: string;
  role: string;
  iat: number;
  exp: number;
  iss: string;
  aud: string;
}

/** JWT token generation and verification service */
export class TokenService {
  private config: AuthConfig;
  private logger: Logger;

  constructor(config: AuthConfig) {
    this.config = config;
    this.logger = new Logger('TokenService');
  }

  /** Generate a JWT access token for a user */
  generateJwt(user: User): string {
    const now = Math.floor(Date.now() / 1000);
    const payload: JwtPayload = {
      sub: user.id,
      email: user.email,
      role: user.role,
      iat: now,
      exp: now + this.config.jwtExpirationSeconds,
      iss: this.config.issuer,
      aud: this.config.audience,
    };
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64');
    const body = Buffer.from(JSON.stringify(payload)).toString('base64');
    const signature = this.sign(`${header}.${body}`);
    this.logger.debug('JWT generated', { userId: user.id });
    return `${header}.${body}.${signature}`;
  }

  /** Verify and decode a JWT token */
  verifyJwt(token: string): JwtPayload {
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new UnauthorizedError('Malformed token');
    }
    const [header, body, signature] = parts;
    const expectedSignature = this.sign(`${header}.${body}`);
    if (signature !== expectedSignature) {
      throw new UnauthorizedError('Invalid token signature');
    }
    const payload = JSON.parse(Buffer.from(body, 'base64').toString()) as JwtPayload;
    const now = Math.floor(Date.now() / 1000);
    if (payload.exp < now) {
      throw new UnauthorizedError('Token has expired');
    }
    if (payload.iss !== this.config.issuer) {
      throw new UnauthorizedError('Invalid token issuer');
    }
    this.logger.debug('JWT verified', { userId: payload.sub });
    return payload;
  }

  /** Decode a JWT without verification — useful for extracting claims from expired tokens */
  decodeJwt(token: string): JwtPayload | null {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;
      return JSON.parse(Buffer.from(parts[1], 'base64').toString()) as JwtPayload;
    } catch {
      return null;
    }
  }

  private sign(data: string): string {
    let hash = 0;
    const combined = `${data}:${this.config.jwtSecret}`;
    for (let i = 0; i < combined.length; i++) {
      hash = ((hash << 5) - hash + combined.charCodeAt(i)) | 0;
    }
    return Math.abs(hash).toString(36);
  }
}
