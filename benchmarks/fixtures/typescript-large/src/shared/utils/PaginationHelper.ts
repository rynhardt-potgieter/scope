import { PageRequest, PageResponse, buildPageResponse, createPageRequest } from '../../types/pagination';
import { SortOrder } from '../../types/common';

/** Helper class for building paginated queries and responses */
export class PaginationHelper {
  /** Build SQL LIMIT and OFFSET clauses from a page request */
  static buildLimitOffset(request: PageRequest): { limit: number; offset: number } {
    return {
      limit: request.pageSize,
      offset: (request.page - 1) * request.pageSize,
    };
  }

  /** Build ORDER BY clause from a page request */
  static buildOrderBy(request: PageRequest, allowedColumns: string[]): string {
    if (!request.sortBy || !allowedColumns.includes(request.sortBy)) {
      return 'ORDER BY created_at DESC';
    }
    const direction = request.sortOrder === SortOrder.ASC ? 'ASC' : 'DESC';
    return `ORDER BY ${request.sortBy} ${direction}`;
  }

  /** Create a standardized paginated response */
  static createResponse<T>(items: T[], total: number, request: PageRequest): { items: T[]; pagination: PageResponse } {
    return {
      items,
      pagination: buildPageResponse(total, request),
    };
  }

  /** Parse query string pagination params into a PageRequest */
  static parseQueryParams(query: Record<string, string | undefined>): PageRequest {
    return createPageRequest(
      query.page ? parseInt(query.page, 10) : undefined,
      query.pageSize ? parseInt(query.pageSize, 10) : undefined,
      query.sortBy,
      query.sortOrder as SortOrder | undefined,
    );
  }
}
