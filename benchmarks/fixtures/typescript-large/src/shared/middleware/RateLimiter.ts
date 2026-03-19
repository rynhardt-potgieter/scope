import { Logger } from '../utils/Logger';
import { RATE_LIMIT_STANDARD } from '../../types/constants';

/** Tracks request counts per client */
interface RateLimitEntry {
  count: number;
  windowStart: number;
}

/** Token-bucket rate limiter keyed by client IP or API key */
export class RateLimiter {
  private limits: Map<string, RateLimitEntry>;
  private maxRequests: number;
  private windowMs: number;
  private logger: Logger;

  constructor(maxRequests: number = RATE_LIMIT_STANDARD, windowMs: number = 60_000) {
    this.limits = new Map();
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
    this.logger = new Logger('RateLimiter');
  }

  /** Check if a client is allowed to make a request; returns true if allowed */
  isAllowed(clientId: string): boolean {
    const now = Date.now();
    const entry = this.limits.get(clientId);

    if (!entry || now - entry.windowStart > this.windowMs) {
      this.limits.set(clientId, { count: 1, windowStart: now });
      return true;
    }

    if (entry.count >= this.maxRequests) {
      this.logger.warn('Rate limit exceeded', { clientId, count: entry.count });
      return false;
    }

    entry.count++;
    return true;
  }

  /** Get remaining requests for a client in the current window */
  remaining(clientId: string): number {
    const entry = this.limits.get(clientId);
    if (!entry || Date.now() - entry.windowStart > this.windowMs) {
      return this.maxRequests;
    }
    return Math.max(0, this.maxRequests - entry.count);
  }

  /** Reset rate limit state for a specific client */
  reset(clientId: string): void {
    this.limits.delete(clientId);
  }

  /** Clean up expired entries */
  cleanup(): number {
    const now = Date.now();
    let removed = 0;
    for (const [key, entry] of this.limits.entries()) {
      if (now - entry.windowStart > this.windowMs) {
        this.limits.delete(key);
        removed++;
      }
    }
    return removed;
  }
}
