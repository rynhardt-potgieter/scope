/** Request to update a user */
export interface UpdateUserRequest {
  firstName?: string;
  lastName?: string;
}

/** User detail response */
export interface UserDetailResponse {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  isActive: boolean;
  createdAt: string;
}

/** User search query parameters */
export interface UserSearchQuery {
  query: string;
  page?: number;
  pageSize?: number;
}
