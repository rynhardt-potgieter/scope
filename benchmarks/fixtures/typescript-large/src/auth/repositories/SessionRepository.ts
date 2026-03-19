import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Session } from '../models/Session';
import { EntityId } from '../../types/common';

/** Repository for session persistence */
export class SessionRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('SessionRepository');
  }

  /** Persist a new session */
  async save(session: Session): Promise<Session> {
    await this.db.execute(
      `INSERT INTO sessions (id, user_id, token, refresh_token, expires_at, ip_address, user_agent, is_revoked, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
      [session.id, session.userId, session.token, session.refreshToken, session.expiresAt, session.ipAddress, session.userAgent, session.isRevoked, session.createdAt],
    );
    this.logger.debug('Session saved', { sessionId: session.id });
    return session;
  }

  /** Find a session by ID */
  async find(sessionId: EntityId): Promise<Session | null> {
    const result = await this.db.query<Session>('SELECT * FROM sessions WHERE id = $1', [sessionId]);
    return result.rows[0] ?? null;
  }

  /** Find a session by refresh token */
  async findByRefreshToken(refreshToken: string): Promise<Session | null> {
    const result = await this.db.query<Session>('SELECT * FROM sessions WHERE refresh_token = $1', [refreshToken]);
    return result.rows[0] ?? null;
  }

  /** Revoke a specific session */
  async revoke(sessionId: EntityId): Promise<void> {
    await this.db.execute('UPDATE sessions SET is_revoked = true WHERE id = $1', [sessionId]);
  }

  /** Revoke all sessions for a user */
  async revokeAllForUser(userId: EntityId): Promise<number> {
    return this.db.execute('UPDATE sessions SET is_revoked = true WHERE user_id = $1', [userId]);
  }

  /** Delete expired sessions older than the cutoff */
  async deleteExpired(): Promise<number> {
    return this.db.execute('DELETE FROM sessions WHERE expires_at < $1', [new Date()]);
  }
}
