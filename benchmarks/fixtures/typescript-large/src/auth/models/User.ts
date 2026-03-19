import { BaseEntity, EntityId } from '../../types/common';
import { UserRole } from '../../types/enums';

/** Core user entity for authentication */
export interface User extends BaseEntity {
  email: string;
  passwordHash: string;
  passwordSalt: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  isActive: boolean;
  lastLoginAt: Date | null;
  failedLoginAttempts: number;
  lockedUntil: Date | null;
  emailVerified: boolean;
  verificationToken: string | null;
}

/** Data needed to create a new user */
export interface CreateUserData {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  role?: UserRole;
}

/** Fields that can be updated on an existing user */
export interface UpdateUserData {
  firstName?: string;
  lastName?: string;
  role?: UserRole;
  isActive?: boolean;
}
