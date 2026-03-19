import { Logger } from '../../shared/utils/Logger';
import { Permission, PermissionAction, PermissionGrant } from '../models/Permission';
import { EntityId } from '../../types/common';
import { ForbiddenError, NotFoundError } from '../../types/errors';
import { DatabaseClient } from '../../shared/database/DatabaseClient';

/** Service for checking and managing granular permissions */
export class PermissionService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('PermissionService');
  }

  /** Check whether a user has a specific permission on a resource */
  async checkPermission(userId: EntityId, resource: string, action: PermissionAction): Promise<boolean> {
    const result = await this.db.query<{ count: number }>(
      `SELECT COUNT(*) as count FROM permission_grants pg
       JOIN permissions p ON pg.permission_id = p.id
       WHERE pg.granted_to = $1 AND p.resource = $2 AND p.action = $3`,
      [userId, resource, action],
    );
    const hasPermission = result.rows.length > 0 && result.rows[0].count > 0;
    this.logger.debug('Permission check', { userId, resource, action, hasPermission });
    return hasPermission;
  }

  /** Grant a permission to a user */
  async grantPermission(permissionId: EntityId, grantedTo: EntityId, grantedBy: EntityId): Promise<PermissionGrant> {
    const grant: PermissionGrant = {
      permissionId,
      grantedTo,
      grantedBy,
      grantedAt: new Date(),
    };
    await this.db.execute(
      'INSERT INTO permission_grants (permission_id, granted_to, granted_by, granted_at) VALUES ($1, $2, $3, $4)',
      [grant.permissionId, grant.grantedTo, grant.grantedBy, grant.grantedAt],
    );
    this.logger.info('Permission granted', { permissionId, grantedTo, grantedBy });
    return grant;
  }

  /** Revoke a permission from a user */
  async revokePermission(permissionId: EntityId, userId: EntityId): Promise<void> {
    const deleted = await this.db.execute(
      'DELETE FROM permission_grants WHERE permission_id = $1 AND granted_to = $2',
      [permissionId, userId],
    );
    if (deleted === 0) {
      throw new NotFoundError('PermissionGrant', `${permissionId}:${userId}`);
    }
    this.logger.info('Permission revoked', { permissionId, userId });
  }

  /** Require a permission, throwing ForbiddenError if not held */
  async requirePermission(userId: EntityId, resource: string, action: PermissionAction): Promise<void> {
    const allowed = await this.checkPermission(userId, resource, action);
    if (!allowed) {
      throw new ForbiddenError(`User ${userId} lacks ${action} permission on ${resource}`);
    }
  }

  /** List all permissions granted to a user */
  async listUserPermissions(userId: EntityId): Promise<Permission[]> {
    const result = await this.db.query<Permission>(
      `SELECT p.* FROM permissions p
       JOIN permission_grants pg ON p.id = pg.permission_id
       WHERE pg.granted_to = $1`,
      [userId],
    );
    return result.rows;
  }
}
