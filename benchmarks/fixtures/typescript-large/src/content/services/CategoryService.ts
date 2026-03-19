import { Logger } from '../../shared/utils/Logger';
import { CategoryRepository } from '../repositories/CategoryRepository';
import { Category } from '../models/Category';
import { EntityId } from '../../types/common';
import { NotFoundError, ValidationError, ConflictError } from '../../types/errors';
import { StringUtils } from '../../shared/utils/StringUtils';

/** Service for managing content categories */
export class CategoryService {
  private categoryRepo: CategoryRepository;
  private logger: Logger;

  constructor(categoryRepo: CategoryRepository) {
    this.categoryRepo = categoryRepo;
    this.logger = new Logger('CategoryService');
  }

  /** Create a new category */
  async createCategory(name: string, description: string, parentId: EntityId | null): Promise<Category> {
    const slug = StringUtils.slugify(name);
    const existing = await this.categoryRepo.findBySlug(slug);
    if (existing) {
      throw new ConflictError(`Category with slug "${slug}" already exists`);
    }

    const category: Category = {
      id: `cat_${Date.now()}`,
      name,
      slug,
      description,
      parentId,
      sortOrder: 0,
      isActive: true,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const saved = await this.categoryRepo.save(category);
    this.logger.info('Category created', { categoryId: saved.id, name });
    return saved;
  }

  /** Get a category by ID */
  async getCategory(categoryId: EntityId): Promise<Category> {
    const category = await this.categoryRepo.findById(categoryId);
    if (!category) throw new NotFoundError('Category', categoryId);
    return category;
  }

  /** List all active categories */
  async listCategories(): Promise<Category[]> {
    return this.categoryRepo.findAll();
  }

  /** Update a category */
  async updateCategory(categoryId: EntityId, data: { name?: string; description?: string }): Promise<Category> {
    const category = await this.categoryRepo.findById(categoryId);
    if (!category) throw new NotFoundError('Category', categoryId);

    const updated: Category = {
      ...category,
      name: data.name ?? category.name,
      slug: data.name ? StringUtils.slugify(data.name) : category.slug,
      description: data.description ?? category.description,
      updatedAt: new Date(),
    };

    await this.categoryRepo.update(updated);
    this.logger.info('Category updated', { categoryId });
    return updated;
  }

  /** Deactivate a category */
  async deactivateCategory(categoryId: EntityId): Promise<void> {
    const category = await this.categoryRepo.findById(categoryId);
    if (!category) throw new NotFoundError('Category', categoryId);
    await this.categoryRepo.update({ ...category, isActive: false, updatedAt: new Date() });
    this.logger.info('Category deactivated', { categoryId });
  }
}
