import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Category } from '../models/Category';
import { EntityId } from '../../types/common';

/** Repository for category persistence */
export class CategoryRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('CategoryRepository');
  }

  /** Save a new category */
  async save(category: Category): Promise<Category> {
    await this.db.execute(
      'INSERT INTO categories (id, name, slug, description, parent_id, sort_order, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)',
      [category.id, category.name, category.slug, category.description, category.parentId, category.sortOrder, category.isActive, category.createdAt],
    );
    return category;
  }

  /** Find a category by ID */
  async findById(categoryId: EntityId): Promise<Category | null> {
    const result = await this.db.query<Category>('SELECT * FROM categories WHERE id = $1', [categoryId]);
    return result.rows[0] ?? null;
  }

  /** Find a category by slug */
  async findBySlug(slug: string): Promise<Category | null> {
    const result = await this.db.query<Category>('SELECT * FROM categories WHERE slug = $1', [slug]);
    return result.rows[0] ?? null;
  }

  /** Find all active categories */
  async findAll(): Promise<Category[]> {
    const result = await this.db.query<Category>('SELECT * FROM categories WHERE is_active = true ORDER BY sort_order');
    return result.rows;
  }

  /** Update a category */
  async update(category: Category): Promise<void> {
    await this.db.execute(
      'UPDATE categories SET name = $1, slug = $2, description = $3, is_active = $4, updated_at = $5 WHERE id = $6',
      [category.name, category.slug, category.description, category.isActive, category.updatedAt, category.id],
    );
  }
}
