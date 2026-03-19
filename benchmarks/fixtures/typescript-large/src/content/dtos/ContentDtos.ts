/** Request to create content */
export interface CreateContentRequest {
  title: string;
  body: string;
  categoryId?: string;
  tags?: string[];
}

/** Request to update content */
export interface UpdateContentRequest {
  title?: string;
  body?: string;
  categoryId?: string;
  tags?: string[];
}

/** Content search query */
export interface ContentSearchQuery {
  query: string;
  categoryId?: string;
  status?: string;
  page?: number;
  pageSize?: number;
}
