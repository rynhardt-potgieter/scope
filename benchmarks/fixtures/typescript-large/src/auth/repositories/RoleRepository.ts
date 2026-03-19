import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Role, CreateRoleData } from '../models/Role';
import { Permission } from '../models/Permission';
import { EntityId } from '../../types/common';

/** Repository for role persistence */
export class RoleRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('RoleRepository');
  }

  /** Find a role by ID with its permissions */
  async findById(roleId: EntityId): Promise<Role | null> {
    const result = await this.db.query<Role>('SELECT * FROM roles WHERE id = $1', [roleId]);
    if (result.rows.length === 0) return null;
    const role = result.rows[0];
    const permissions = await this.db.query<Permission>(
      'SELECT p.* FROM permissions p JOIN role_permissions rp ON p.id = rp.permission_id WHERE rp.role_id = $1',
      [roleId],
    );
    return { ...role, permissions: permissions.rows };
  }

  /** Find a role by name */
  async findByName(name: string): Promise<Role | null> {
    const result = await this.db.query<Role>('SELECT * FROM roles WHERE name = $1', [name]);
    return result.rows[0] ?? null;
  }

  /** List all roles */
  async findAll(): Promise<Role[]> {
    const result = await this.db.query<Role>('SELECT * FROM roles ORDER BY name');
    return result.rows;
  }

  /** Create a new role */
  async create(data: CreateRoleData): Promise<Role> {
    const id = `role_${Date.now()}`;
    await this.db.execute(
      'INSERT INTO roles (id, name, description, is_system, created_at) VALUES ($1, $2, $3, false, $4)',
      [id, data.name, data.description, new Date()],
    );
    this.logger.info('Role created', { roleId: id, name: data.name });
    return this.findById(id) as Promise<Role>;
  }
}
