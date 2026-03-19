import { Logger } from '../../shared/utils/Logger';
import { CryptoService } from '../../shared/utils/Crypto';
import { UserRepository } from '../repositories/UserRepository';
import { SessionService } from './SessionService';
import { TokenService } from './TokenService';
import { User, CreateUserData } from '../models/User';
import { Session } from '../models/Session';
import { UnauthorizedError, ValidationError, ConflictError } from '../../types/errors';
import { UserRole } from '../../types/enums';
import { EntityId } from '../../types/common';

/** Authentication result containing tokens and user info */
export interface AuthResult {
  user: Omit<User, 'passwordHash' | 'passwordSalt'>;
  accessToken: string;
  refreshToken: string;
  expiresAt: Date;
}

/** Core authentication service handling login, registration, and token refresh */
export class AuthService {
  private userRepo: UserRepository;
  private sessionService: SessionService;
  private tokenService: TokenService;
  private crypto: CryptoService;
  private logger: Logger;

  constructor(
    userRepo: UserRepository,
    sessionService: SessionService,
    tokenService: TokenService,
    crypto: CryptoService,
  ) {
    this.userRepo = userRepo;
    this.sessionService = sessionService;
    this.tokenService = tokenService;
    this.crypto = crypto;
    this.logger = new Logger('AuthService');
  }

  /** Authenticate a user with email and password */
  async login(email: string, password: string, ip: string, userAgent: string): Promise<AuthResult> {
    const user = await this.userRepo.findByEmail(email);
    if (!user) {
      this.logger.warn('Login attempt for unknown email', { email });
      throw new UnauthorizedError('Invalid email or password');
    }

    if (!user.isActive) {
      throw new UnauthorizedError('Account is disabled');
    }

    if (user.lockedUntil && user.lockedUntil > new Date()) {
      throw new UnauthorizedError('Account is temporarily locked');
    }

    const isValid = await this.crypto.verify(password, user.passwordHash, user.passwordSalt);
    if (!isValid) {
      this.logger.warn('Failed login attempt', { userId: user.id });
      await this.userRepo.incrementFailedAttempts(user.id);
      throw new UnauthorizedError('Invalid email or password');
    }

    await this.userRepo.resetFailedAttempts(user.id);
    const session = await this.sessionService.createSession({
      userId: user.id,
      ipAddress: ip,
      userAgent,
    });
    const accessToken = this.tokenService.generateJwt(user);
    this.logger.info('User logged in', { userId: user.id });

    return {
      user: this.stripSensitiveFields(user),
      accessToken,
      refreshToken: session.refreshToken,
      expiresAt: session.expiresAt,
    };
  }

  /** Register a new user account */
  async register(data: CreateUserData): Promise<AuthResult> {
    const existing = await this.userRepo.findByEmail(data.email);
    if (existing) {
      throw new ConflictError('Email already registered');
    }

    const { hash, salt } = await this.crypto.hash(data.password);
    const user = await this.userRepo.create({
      ...data,
      passwordHash: hash,
      passwordSalt: salt,
      role: data.role ?? UserRole.VIEWER,
    });

    const session = await this.sessionService.createSession({
      userId: user.id,
      ipAddress: '0.0.0.0',
      userAgent: 'registration',
    });
    const accessToken = this.tokenService.generateJwt(user);
    this.logger.info('User registered', { userId: user.id });

    return {
      user: this.stripSensitiveFields(user),
      accessToken,
      refreshToken: session.refreshToken,
      expiresAt: session.expiresAt,
    };
  }

  /** Refresh an expired access token using a valid refresh token */
  async refreshToken(refreshToken: string): Promise<{ accessToken: string; expiresAt: Date }> {
    const session = await this.sessionService.findByRefreshToken(refreshToken);
    if (!session || session.isRevoked) {
      throw new UnauthorizedError('Invalid refresh token');
    }

    const user = await this.userRepo.findById(session.userId);
    if (!user || !user.isActive) {
      throw new UnauthorizedError('User not found or disabled');
    }

    const accessToken = this.tokenService.generateJwt(user);
    this.logger.info('Token refreshed', { userId: user.id });
    return { accessToken, expiresAt: session.expiresAt };
  }

  /** Validate that a session is still active */
  async validateSession(sessionId: EntityId): Promise<Session | null> {
    return this.sessionService.getSession(sessionId);
  }

  /** Log out by revoking the session */
  async logout(sessionId: EntityId): Promise<void> {
    await this.sessionService.destroySession(sessionId);
    this.logger.info('User logged out', { sessionId });
  }

  private stripSensitiveFields(user: User): Omit<User, 'passwordHash' | 'passwordSalt'> {
    const { passwordHash, passwordSalt, ...safe } = user;
    return safe;
  }
}
