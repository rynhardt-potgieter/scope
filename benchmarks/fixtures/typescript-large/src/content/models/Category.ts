import { BaseEntity, EntityId } from '../../types/common';

/** Content category with hierarchical structure */
export interface Category extends BaseEntity {
  name: string;
  slug: string;
  description: string;
  parentId: EntityId | null;
  sortOrder: number;
  isActive: boolean;
}
