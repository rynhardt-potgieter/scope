/** Standard API response wrapper */
export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: Date;
}

/** Paginated response with metadata */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

/** Sort direction for list queries */
export enum SortOrder {
  ASC = 'ASC',
  DESC = 'DESC',
}

/** Generic ID type alias */
export type EntityId = string;

/** Timestamp fields shared across all entities */
export interface Timestamped {
  createdAt: Date;
  updatedAt: Date;
}

/** Base entity with common fields */
export interface BaseEntity extends Timestamped {
  id: EntityId;
}

/** Result type for operations that may fail gracefully */
export interface OperationResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}
