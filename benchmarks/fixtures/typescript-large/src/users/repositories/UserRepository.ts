import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { User } from '../../auth/models/User';
import { EntityId } from '../../types/common';
import { PageRequest } from '../../types/pagination';

/** User repository for the users domain (delegates to shared DB) */
export class UserRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('Users.UserRepository');
  }

  /** Find a user by their unique ID */
  async findById(id: EntityId): Promise<User | null> {
    const result = await this.db.query<User>('SELECT * FROM users WHERE id = $1', [id]);
    return result.rows[0] ?? null;
  }

  /** Find a user by email */
  async findByEmail(email: string): Promise<User | null> {
    const result = await this.db.query<User>('SELECT * FROM users WHERE email = $1', [email]);
    return result.rows[0] ?? null;
  }

  /** Update user fields */
  async update(id: EntityId, data: Partial<User>): Promise<User | null> {
    const existing = await this.findById(id);
    if (!existing) return null;
    const updated = { ...existing, ...data, updatedAt: new Date() };
    await this.db.execute(
      'UPDATE users SET first_name = $1, last_name = $2, updated_at = $3 WHERE id = $4',
      [updated.firstName, updated.lastName, updated.updatedAt, id],
    );
    return updated;
  }

  /** Soft-delete a user */
  async delete(id: EntityId): Promise<boolean> {
    const affected = await this.db.execute('UPDATE users SET is_active = false WHERE id = $1', [id]);
    return affected > 0;
  }

  /** Search users by name or email */
  async search(query: string, pageRequest: PageRequest): Promise<{ users: User[]; total: number }> {
    const offset = (pageRequest.page - 1) * pageRequest.pageSize;
    const result = await this.db.query<User>(
      'SELECT * FROM users WHERE first_name ILIKE $1 OR last_name ILIKE $1 OR email ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3',
      [`%${query}%`, pageRequest.pageSize, offset],
    );
    const countResult = await this.db.query<{ count: number }>(
      'SELECT COUNT(*) as count FROM users WHERE first_name ILIKE $1 OR last_name ILIKE $1 OR email ILIKE $1',
      [`%${query}%`],
    );
    const total = countResult.rows[0]?.count ?? 0;
    this.logger.debug('User search executed', { query, resultCount: result.rows.length });
    return { users: result.rows, total };
  }
}
