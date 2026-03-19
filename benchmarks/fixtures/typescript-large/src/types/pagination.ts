import { SortOrder } from './common';

/** Inbound pagination request from the client */
export interface PageRequest {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: SortOrder;
}

/** Outbound pagination metadata in responses */
export interface PageResponse {
  currentPage: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

/** Creates a PageRequest with safe defaults */
export function createPageRequest(
  page?: number,
  pageSize?: number,
  sortBy?: string,
  sortOrder?: SortOrder,
): PageRequest {
  const safePage = Math.max(1, page ?? 1);
  const safeSize = Math.min(100, Math.max(1, pageSize ?? 20));
  return { page: safePage, pageSize: safeSize, sortBy, sortOrder };
}

/** Builds a PageResponse from total count and page request */
export function buildPageResponse(total: number, request: PageRequest): PageResponse {
  const totalPages = Math.ceil(total / request.pageSize);
  return {
    currentPage: request.page,
    pageSize: request.pageSize,
    totalItems: total,
    totalPages,
    hasNext: request.page < totalPages,
    hasPrevious: request.page > 1,
  };
}
