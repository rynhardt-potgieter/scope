import { BaseEntity, EntityId } from '../../types/common';
import { Permission } from './Permission';

/** Named role with a set of permissions */
export interface Role extends BaseEntity {
  name: string;
  description: string;
  permissions: Permission[];
  isSystem: boolean;
}

/** Data needed to create a new role */
export interface CreateRoleData {
  name: string;
  description: string;
  permissionIds: EntityId[];
}

/** Fields that can be updated on a role */
export interface UpdateRoleData {
  name?: string;
  description?: string;
  permissionIds?: EntityId[];
}
