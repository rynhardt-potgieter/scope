import { Currency } from './money';

/** Maximum number of retry attempts for failed payments */
export const MAX_RETRY_ATTEMPTS = 3;

/** Default currency for new accounts */
export const DEFAULT_CURRENCY: Currency = Currency.USD;

/** JWT token expiration in seconds (1 hour) */
export const JWT_EXPIRATION_SECONDS = 3600;

/** Refresh token expiration in seconds (30 days) */
export const REFRESH_TOKEN_EXPIRATION_SECONDS = 2592000;

/** Session idle timeout in seconds (30 minutes) */
export const SESSION_IDLE_TIMEOUT_SECONDS = 1800;

/** Default page size for paginated queries */
export const DEFAULT_PAGE_SIZE = 20;

/** Maximum page size allowed */
export const MAX_PAGE_SIZE = 100;

/** Rate limit: requests per minute for standard users */
export const RATE_LIMIT_STANDARD = 60;

/** Rate limit: requests per minute for API users */
export const RATE_LIMIT_API = 300;

/** Maximum file upload size in bytes (10 MB) */
export const MAX_UPLOAD_SIZE_BYTES = 10 * 1024 * 1024;

/** Allowed image MIME types for uploads */
export const ALLOWED_IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif'];

/** Cache TTL in seconds for user profile data */
export const USER_CACHE_TTL_SECONDS = 600;

/** Cache TTL in seconds for content data */
export const CONTENT_CACHE_TTL_SECONDS = 300;

/** Notification delivery timeout in milliseconds */
export const NOTIFICATION_DELIVERY_TIMEOUT_MS = 5000;

/** Retry delay base in milliseconds (used for exponential backoff) */
export const RETRY_DELAY_BASE_MS = 1000;

/** Maximum content title length */
export const MAX_TITLE_LENGTH = 200;

/** Maximum content body length */
export const MAX_BODY_LENGTH = 50000;
