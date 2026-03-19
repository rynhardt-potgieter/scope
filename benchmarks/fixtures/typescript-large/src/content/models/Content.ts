import { BaseEntity, EntityId } from '../../types/common';
import { ContentStatus } from '../../types/enums';

/** A content item (article, page, post) */
export interface Content extends BaseEntity {
  title: string;
  slug: string;
  body: string;
  excerpt: string;
  authorId: EntityId;
  categoryId: EntityId | null;
  tags: string[];
  status: ContentStatus;
  publishedAt: Date | null;
  featuredImageUrl: string | null;
  viewCount: number;
  metadata: Record<string, unknown>;
}
