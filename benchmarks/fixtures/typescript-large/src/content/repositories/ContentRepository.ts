import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Content } from '../models/Content';
import { EntityId } from '../../types/common';
import { ContentStatus } from '../../types/enums';
import { PageRequest } from '../../types/pagination';

/** Repository for content persistence */
export class ContentRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('ContentRepository');
  }

  /** Save a new content item */
  async save(content: Content): Promise<Content> {
    await this.db.execute(
      `INSERT INTO content (id, title, slug, body, excerpt, author_id, category_id, status, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`,
      [content.id, content.title, content.slug, content.body, content.excerpt, content.authorId, content.categoryId, content.status, content.createdAt, content.updatedAt],
    );
    this.logger.debug('Content saved', { contentId: content.id });
    return content;
  }

  /** Find content by ID */
  async findById(contentId: EntityId): Promise<Content | null> {
    const result = await this.db.query<Content>('SELECT * FROM content WHERE id = $1', [contentId]);
    return result.rows[0] ?? null;
  }

  /** Find all content with optional status filter and pagination */
  async findAll(pageRequest: PageRequest, status?: ContentStatus): Promise<{ items: Content[]; total: number }> {
    const offset = (pageRequest.page - 1) * pageRequest.pageSize;
    let sql = 'SELECT * FROM content';
    const params: unknown[] = [];

    if (status) {
      sql += ' WHERE status = $1';
      params.push(status);
    }
    sql += ' ORDER BY created_at DESC LIMIT $' + (params.length + 1) + ' OFFSET $' + (params.length + 2);
    params.push(pageRequest.pageSize, offset);

    const result = await this.db.query<Content>(sql, params);
    const countResult = await this.db.query<{ count: number }>(
      status ? 'SELECT COUNT(*) as count FROM content WHERE status = $1' : 'SELECT COUNT(*) as count FROM content',
      status ? [status] : [],
    );
    return { items: result.rows, total: countResult.rows[0]?.count ?? 0 };
  }

  /** Update a content item */
  async update(content: Content): Promise<void> {
    await this.db.execute(
      'UPDATE content SET title = $1, slug = $2, body = $3, excerpt = $4, status = $5, updated_at = $6 WHERE id = $7',
      [content.title, content.slug, content.body, content.excerpt, content.status, content.updatedAt, content.id],
    );
  }
}
