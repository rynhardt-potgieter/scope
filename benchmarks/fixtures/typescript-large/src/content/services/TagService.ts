import { Logger } from '../../shared/utils/Logger';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Tag } from '../models/Tag';
import { StringUtils } from '../../shared/utils/StringUtils';
import { ConflictError, NotFoundError } from '../../types/errors';
import { EntityId } from '../../types/common';

/** Service for managing content tags */
export class TagService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('TagService');
  }

  /** Create a new tag */
  async createTag(name: string): Promise<Tag> {
    const slug = StringUtils.slugify(name);
    const existing = await this.findBySlug(slug);
    if (existing) throw new ConflictError(`Tag "${name}" already exists`);

    const tag: Tag = {
      id: `tag_${Date.now()}`,
      name,
      slug,
      usageCount: 0,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    await this.db.execute('INSERT INTO tags (id, name, slug, usage_count, created_at) VALUES ($1,$2,$3,$4,$5)',
      [tag.id, tag.name, tag.slug, tag.usageCount, tag.createdAt]);
    this.logger.info('Tag created', { tagId: tag.id, name });
    return tag;
  }

  /** Find a tag by slug */
  async findBySlug(slug: string): Promise<Tag | null> {
    const result = await this.db.query<Tag>('SELECT * FROM tags WHERE slug = $1', [slug]);
    return result.rows[0] ?? null;
  }

  /** Get popular tags ordered by usage count */
  async getPopular(limit: number = 20): Promise<Tag[]> {
    const result = await this.db.query<Tag>('SELECT * FROM tags ORDER BY usage_count DESC LIMIT $1', [limit]);
    return result.rows;
  }

  /** Increment the usage count of a tag */
  async incrementUsage(tagId: EntityId): Promise<void> {
    await this.db.execute('UPDATE tags SET usage_count = usage_count + 1 WHERE id = $1', [tagId]);
  }

  /** Delete a tag if it has no content references */
  async deleteTag(tagId: EntityId): Promise<void> {
    const result = await this.db.query<{ count: number }>('SELECT COUNT(*) as count FROM content_tags WHERE tag_id = $1', [tagId]);
    if (result.rows[0]?.count > 0) {
      throw new ConflictError('Cannot delete tag that is in use');
    }
    await this.db.execute('DELETE FROM tags WHERE id = $1', [tagId]);
    this.logger.info('Tag deleted', { tagId });
  }
}
