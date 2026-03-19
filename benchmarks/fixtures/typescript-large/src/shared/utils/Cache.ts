import { Logger } from './Logger';

/** Entry stored in the cache with expiration metadata */
interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

/** In-memory cache with TTL-based expiration */
export class CacheService {
  private store: Map<string, CacheEntry<unknown>>;
  private logger: Logger;
  private defaultTtlMs: number;

  constructor(defaultTtlMs: number = 300_000) {
    this.store = new Map();
    this.logger = new Logger('CacheService');
    this.defaultTtlMs = defaultTtlMs;
  }

  /** Retrieve a value from cache, or undefined if missing/expired */
  get<T>(key: string): T | undefined {
    const entry = this.store.get(key) as CacheEntry<T> | undefined;
    if (!entry) {
      return undefined;
    }
    if (Date.now() > entry.expiresAt) {
      this.store.delete(key);
      this.logger.debug('Cache entry expired', { key });
      return undefined;
    }
    this.logger.debug('Cache hit', { key });
    return entry.value;
  }

  /** Store a value in cache with optional custom TTL */
  set<T>(key: string, value: T, ttlMs?: number): void {
    const expiresAt = Date.now() + (ttlMs ?? this.defaultTtlMs);
    this.store.set(key, { value, expiresAt });
    this.logger.debug('Cache set', { key, ttlMs: ttlMs ?? this.defaultTtlMs });
  }

  /** Remove a specific key from cache */
  invalidate(key: string): void {
    this.store.delete(key);
    this.logger.debug('Cache invalidated', { key });
  }

  /** Remove all keys matching a prefix */
  invalidatePrefix(prefix: string): void {
    let count = 0;
    for (const key of this.store.keys()) {
      if (key.startsWith(prefix)) {
        this.store.delete(key);
        count++;
      }
    }
    this.logger.debug('Cache prefix invalidated', { prefix, count });
  }

  /** Remove all expired entries */
  cleanup(): number {
    const now = Date.now();
    let removed = 0;
    for (const [key, entry] of this.store.entries()) {
      if (now > entry.expiresAt) {
        this.store.delete(key);
        removed++;
      }
    }
    this.logger.info('Cache cleanup complete', { removed });
    return removed;
  }

  /** Get current cache size */
  size(): number {
    return this.store.size;
  }
}
