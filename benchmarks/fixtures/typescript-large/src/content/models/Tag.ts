import { BaseEntity } from '../../types/common';

/** Tag for categorizing content */
export interface Tag extends BaseEntity {
  name: string;
  slug: string;
  usageCount: number;
}
