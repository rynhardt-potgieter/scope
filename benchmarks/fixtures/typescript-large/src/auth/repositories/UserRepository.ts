import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { User, CreateUserData } from '../models/User';
import { EntityId } from '../../types/common';
import { UserRole } from '../../types/enums';
import { CryptoService } from '../../shared/utils/Crypto';

/** Repository for user persistence operations */
export class UserRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('UserRepository');
  }

  /** Find a user by their unique ID */
  async findById(id: EntityId): Promise<User | null> {
    const result = await this.db.query<User>('SELECT * FROM users WHERE id = $1', [id]);
    return result.rows[0] ?? null;
  }

  /** Find a user by email address */
  async findByEmail(email: string): Promise<User | null> {
    const result = await this.db.query<User>('SELECT * FROM users WHERE email = $1', [email]);
    return result.rows[0] ?? null;
  }

  /** Create a new user record */
  async create(data: CreateUserData & { passwordHash: string; passwordSalt: string; role: UserRole }): Promise<User> {
    const crypto = new CryptoService();
    const id = crypto.generateToken(16);
    const now = new Date();
    const user: User = {
      id,
      email: data.email,
      passwordHash: data.passwordHash,
      passwordSalt: data.passwordSalt,
      firstName: data.firstName,
      lastName: data.lastName,
      role: data.role,
      isActive: true,
      lastLoginAt: null,
      failedLoginAttempts: 0,
      lockedUntil: null,
      emailVerified: false,
      verificationToken: crypto.generateToken(32),
      createdAt: now,
      updatedAt: now,
    };
    await this.db.execute(
      `INSERT INTO users (id, email, password_hash, password_salt, first_name, last_name, role, is_active, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`,
      [user.id, user.email, user.passwordHash, user.passwordSalt, user.firstName, user.lastName, user.role, user.isActive, user.createdAt, user.updatedAt],
    );
    this.logger.info('User created', { userId: user.id });
    return user;
  }

  /** Update an existing user record */
  async update(id: EntityId, data: Partial<User>): Promise<User | null> {
    const existing = await this.findById(id);
    if (!existing) return null;
    const updated = { ...existing, ...data, updatedAt: new Date() };
    await this.db.execute('UPDATE users SET first_name = $1, last_name = $2, role = $3, updated_at = $4 WHERE id = $5',
      [updated.firstName, updated.lastName, updated.role, updated.updatedAt, id]);
    return updated;
  }

  /** Soft-delete a user by deactivating their account */
  async delete(id: EntityId): Promise<boolean> {
    const affected = await this.db.execute('UPDATE users SET is_active = false, updated_at = $1 WHERE id = $2', [new Date(), id]);
    return affected > 0;
  }

  /** Increment the failed login attempt counter */
  async incrementFailedAttempts(userId: EntityId): Promise<void> {
    await this.db.execute('UPDATE users SET failed_login_attempts = failed_login_attempts + 1 WHERE id = $1', [userId]);
  }

  /** Reset the failed login attempt counter after a successful login */
  async resetFailedAttempts(userId: EntityId): Promise<void> {
    await this.db.execute('UPDATE users SET failed_login_attempts = 0, last_login_at = $1 WHERE id = $2', [new Date(), userId]);
  }
}
