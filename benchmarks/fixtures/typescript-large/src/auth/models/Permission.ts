import { BaseEntity, EntityId } from '../../types/common';

/** A granular permission that can be assigned to roles */
export interface Permission extends BaseEntity {
  name: string;
  description: string;
  resource: string;
  action: PermissionAction;
}

/** Allowed actions on a resource */
export enum PermissionAction {
  CREATE = 'create',
  READ = 'read',
  UPDATE = 'update',
  DELETE = 'delete',
  LIST = 'list',
  MANAGE = 'manage',
}

/** Assignment of a permission to a user or role */
export interface PermissionGrant {
  permissionId: EntityId;
  grantedTo: EntityId;
  grantedBy: EntityId;
  grantedAt: Date;
}
