import { Logger } from '../../shared/utils/Logger';
import { ContentService } from '../../content/services/ContentService';
import { SearchService } from '../../content/services/SearchService';
import { ContentMapper } from '../../content/mappers/ContentMapper';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse, PaginatedResponse } from '../../types/common';
import { ContentDto, ContentDetailDto } from '../../content/mappers/ContentMapper';
import { CreateContentRequest, UpdateContentRequest, ContentSearchQuery } from '../../content/dtos/ContentDtos';
import { createPageRequest } from '../../types/pagination';

/** Controller for content management endpoints */
export class ContentController {
  private contentService: ContentService;
  private searchService: SearchService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(contentService: ContentService, searchService: SearchService, authGuard: AuthGuard) {
    this.contentService = contentService;
    this.searchService = searchService;
    this.authGuard = authGuard;
    this.logger = new Logger('ContentController');
  }

  /** POST /content */
  async create(authHeader: string, request: CreateContentRequest): Promise<ApiResponse<ContentDetailDto>> {
    const user = this.authGuard.guard(authHeader);
    const content = await this.contentService.createContent(
      user.sub,
      request.title,
      request.body,
      request.categoryId ?? null,
      request.tags ?? [],
    );
    return {
      success: true,
      data: ContentMapper.toDetailDto(content),
      message: 'Content created',
      timestamp: new Date(),
    };
  }

  /** PUT /content/:id */
  async update(authHeader: string, contentId: string, request: UpdateContentRequest): Promise<ApiResponse<ContentDetailDto>> {
    const user = this.authGuard.guard(authHeader);
    const updated = await this.contentService.updateContent(contentId, user.sub, request);
    return {
      success: true,
      data: ContentMapper.toDetailDto(updated),
      message: 'Content updated',
      timestamp: new Date(),
    };
  }

  /** DELETE /content/:id */
  async delete(authHeader: string, contentId: string): Promise<ApiResponse<{ deleted: boolean }>> {
    const user = this.authGuard.guard(authHeader);
    await this.contentService.deleteContent(contentId, user.sub);
    return {
      success: true,
      data: { deleted: true },
      message: 'Content deleted',
      timestamp: new Date(),
    };
  }

  /** GET /content/search */
  async search(query: ContentSearchQuery): Promise<ApiResponse<ContentDto[]>> {
    const results = await this.searchService.search(query.query, query.pageSize ?? 20);
    return {
      success: true,
      data: [],
      message: `Found ${results.length} results`,
      timestamp: new Date(),
    };
  }
}
