import { BaseEntity, EntityId } from '../../types/common';

/** Active user session */
export interface Session extends BaseEntity {
  userId: EntityId;
  token: string;
  refreshToken: string;
  expiresAt: Date;
  ipAddress: string;
  userAgent: string;
  isRevoked: boolean;
}

/** Data needed to create a new session */
export interface CreateSessionData {
  userId: EntityId;
  ipAddress: string;
  userAgent: string;
}
