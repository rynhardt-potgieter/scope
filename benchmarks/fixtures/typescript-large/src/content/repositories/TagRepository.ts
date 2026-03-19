import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Tag } from '../models/Tag';
import { EntityId } from '../../types/common';

/** Repository for tag persistence */
export class TagRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('TagRepository');
  }

  /** Find a tag by ID */
  async findById(tagId: EntityId): Promise<Tag | null> {
    const result = await this.db.query<Tag>('SELECT * FROM tags WHERE id = $1', [tagId]);
    return result.rows[0] ?? null;
  }

  /** Find tags by content ID */
  async findByContentId(contentId: EntityId): Promise<Tag[]> {
    const result = await this.db.query<Tag>(
      'SELECT t.* FROM tags t JOIN content_tags ct ON t.id = ct.tag_id WHERE ct.content_id = $1',
      [contentId],
    );
    return result.rows;
  }

  /** Save a tag */
  async save(tag: Tag): Promise<Tag> {
    await this.db.execute('INSERT INTO tags (id, name, slug, usage_count, created_at) VALUES ($1,$2,$3,$4,$5)',
      [tag.id, tag.name, tag.slug, tag.usageCount, tag.createdAt]);
    return tag;
  }

  /** Associate a tag with content */
  async linkToContent(tagId: EntityId, contentId: EntityId): Promise<void> {
    await this.db.execute('INSERT INTO content_tags (tag_id, content_id) VALUES ($1, $2)', [tagId, contentId]);
  }

  /** Remove tag-content association */
  async unlinkFromContent(tagId: EntityId, contentId: EntityId): Promise<void> {
    await this.db.execute('DELETE FROM content_tags WHERE tag_id = $1 AND content_id = $2', [tagId, contentId]);
  }
}
