import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';
import { ContentRepository } from '../repositories/ContentRepository';
import { SearchService } from './SearchService';
import { UserService } from '../../users/services/UserService';
import { Content } from '../models/Content';
import { EntityId } from '../../types/common';
import { ContentStatus } from '../../types/enums';
import { PageRequest, PageResponse, buildPageResponse } from '../../types/pagination';
import { NotFoundError, ValidationError, ForbiddenError } from '../../types/errors';
import { StringUtils } from '../../shared/utils/StringUtils';
import { CONTENT_CACHE_TTL_SECONDS, MAX_TITLE_LENGTH, MAX_BODY_LENGTH } from '../../types/constants';

/** Service for content lifecycle management */
export class ContentService {
  private contentRepo: ContentRepository;
  private searchService: SearchService;
  private userService: UserService;
  private cache: CacheService;
  private logger: Logger;

  constructor(
    contentRepo: ContentRepository,
    searchService: SearchService,
    userService: UserService,
    cache: CacheService,
  ) {
    this.contentRepo = contentRepo;
    this.searchService = searchService;
    this.userService = userService;
    this.cache = cache;
    this.logger = new Logger('ContentService');
  }

  /** Create a new content item */
  async createContent(
    authorId: EntityId,
    title: string,
    body: string,
    categoryId: EntityId | null,
    tags: string[],
  ): Promise<Content> {
    await this.userService.getUser(authorId);

    if (title.length > MAX_TITLE_LENGTH) {
      throw new ValidationError('Title too long', { title: `Maximum ${MAX_TITLE_LENGTH} characters` });
    }
    if (body.length > MAX_BODY_LENGTH) {
      throw new ValidationError('Body too long', { body: `Maximum ${MAX_BODY_LENGTH} characters` });
    }

    const slug = StringUtils.slugify(title);
    const excerpt = StringUtils.truncate(body.replace(/<[^>]*>/g, ''), 200);

    const content: Content = {
      id: `cnt_${Date.now()}`,
      title,
      slug,
      body,
      excerpt,
      authorId,
      categoryId,
      tags,
      status: ContentStatus.DRAFT,
      publishedAt: null,
      featuredImageUrl: null,
      viewCount: 0,
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const saved = await this.contentRepo.save(content);
    await this.searchService.indexContent(saved);
    this.logger.info('Content created', { contentId: saved.id, title });
    return saved;
  }

  /** Update an existing content item */
  async updateContent(
    contentId: EntityId,
    userId: EntityId,
    data: { title?: string; body?: string; categoryId?: EntityId; tags?: string[] },
  ): Promise<Content> {
    const content = await this.contentRepo.findById(contentId);
    if (!content) {
      throw new NotFoundError('Content', contentId);
    }
    if (content.authorId !== userId) {
      throw new ForbiddenError('Only the author can edit this content');
    }

    const updated: Content = {
      ...content,
      title: data.title ?? content.title,
      slug: data.title ? StringUtils.slugify(data.title) : content.slug,
      body: data.body ?? content.body,
      excerpt: data.body ? StringUtils.truncate(data.body.replace(/<[^>]*>/g, ''), 200) : content.excerpt,
      categoryId: data.categoryId ?? content.categoryId,
      tags: data.tags ?? content.tags,
      updatedAt: new Date(),
    };

    await this.contentRepo.update(updated);
    await this.searchService.indexContent(updated);
    this.cache.invalidate(`content:${contentId}`);
    this.logger.info('Content updated', { contentId });
    return updated;
  }

  /** Publish a draft content item */
  async publishContent(contentId: EntityId, userId: EntityId): Promise<Content> {
    const content = await this.contentRepo.findById(contentId);
    if (!content) throw new NotFoundError('Content', contentId);
    if (content.authorId !== userId) throw new ForbiddenError('Only the author can publish');
    if (content.status !== ContentStatus.DRAFT && content.status !== ContentStatus.IN_REVIEW) {
      throw new ValidationError(`Cannot publish content with status ${content.status}`);
    }

    const published = { ...content, status: ContentStatus.PUBLISHED, publishedAt: new Date(), updatedAt: new Date() };
    await this.contentRepo.update(published);
    this.cache.invalidate(`content:${contentId}`);
    this.logger.info('Content published', { contentId });
    return published;
  }

  /** Get content by ID with caching */
  async getContent(contentId: EntityId): Promise<Content> {
    const cached = this.cache.get<Content>(`content:${contentId}`);
    if (cached) return cached;

    const content = await this.contentRepo.findById(contentId);
    if (!content) throw new NotFoundError('Content', contentId);
    this.cache.set(`content:${contentId}`, content, CONTENT_CACHE_TTL_SECONDS * 1000);
    return content;
  }

  /** List content with pagination */
  async listContent(pageRequest: PageRequest, status?: ContentStatus): Promise<{ items: Content[]; pagination: PageResponse }> {
    const { items, total } = await this.contentRepo.findAll(pageRequest, status);
    return { items, pagination: buildPageResponse(total, pageRequest) };
  }

  /** Delete content (soft-delete by setting status to DELETED) */
  async deleteContent(contentId: EntityId, userId: EntityId): Promise<void> {
    const content = await this.contentRepo.findById(contentId);
    if (!content) throw new NotFoundError('Content', contentId);
    if (content.authorId !== userId) throw new ForbiddenError('Only the author can delete');

    await this.contentRepo.update({ ...content, status: ContentStatus.DELETED, updatedAt: new Date() });
    await this.searchService.removeContent(contentId);
    this.cache.invalidate(`content:${contentId}`);
    this.logger.info('Content deleted', { contentId });
  }
}
