import { Logger } from '../../shared/utils/Logger';
import { CryptoService } from '../../shared/utils/Crypto';
import { SessionRepository } from '../repositories/SessionRepository';
import { Session, CreateSessionData } from '../models/Session';
import { EntityId } from '../../types/common';
import { DateUtils } from '../../shared/utils/DateUtils';
import { SESSION_IDLE_TIMEOUT_SECONDS } from '../../types/constants';

/** Manages user sessions — creation, validation, and destruction */
export class SessionService {
  private sessionRepo: SessionRepository;
  private crypto: CryptoService;
  private logger: Logger;

  constructor(sessionRepo: SessionRepository, crypto: CryptoService) {
    this.sessionRepo = sessionRepo;
    this.crypto = crypto;
    this.logger = new Logger('SessionService');
  }

  /** Create a new session for an authenticated user */
  async createSession(data: CreateSessionData): Promise<Session> {
    const token = this.crypto.generateToken(48);
    const refreshToken = this.crypto.generateToken(64);
    const expiresAt = DateUtils.addHours(new Date(), 24);

    const session = await this.sessionRepo.save({
      id: this.crypto.generateToken(16),
      userId: data.userId,
      token,
      refreshToken,
      expiresAt,
      ipAddress: data.ipAddress,
      userAgent: data.userAgent,
      isRevoked: false,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    this.logger.info('Session created', { userId: data.userId });
    return session;
  }

  /** Destroy (revoke) a session */
  async destroySession(sessionId: EntityId): Promise<void> {
    await this.sessionRepo.revoke(sessionId);
    this.logger.info('Session destroyed', { sessionId });
  }

  /** Get a session by ID, returning null if expired or revoked */
  async getSession(sessionId: EntityId): Promise<Session | null> {
    const session = await this.sessionRepo.find(sessionId);
    if (!session) return null;

    if (session.isRevoked || DateUtils.isExpired(session.expiresAt)) {
      this.logger.debug('Session expired or revoked', { sessionId });
      return null;
    }
    return session;
  }

  /** Find a session by its refresh token */
  async findByRefreshToken(refreshToken: string): Promise<Session | null> {
    return this.sessionRepo.findByRefreshToken(refreshToken);
  }

  /** Revoke all sessions for a user (e.g., password change) */
  async revokeAllForUser(userId: EntityId): Promise<number> {
    const count = await this.sessionRepo.revokeAllForUser(userId);
    this.logger.info('All sessions revoked for user', { userId, count });
    return count;
  }

  /** Clean up expired sessions */
  async cleanupExpired(): Promise<number> {
    const count = await this.sessionRepo.deleteExpired();
    this.logger.info('Expired sessions cleaned up', { count });
    return count;
  }
}
