import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';

/** Background worker for periodic cache cleanup */
export class CacheCleanupWorker {
  private cache: CacheService;
  private logger: Logger;

  constructor(cache: CacheService) {
    this.cache = cache;
    this.logger = new Logger('CacheCleanupWorker');
  }

  /** Run cache cleanup, removing all expired entries */
  async cleanupExpiredEntries(): Promise<number> {
    this.logger.info('Starting cache cleanup');
    const removed = this.cache.cleanup();
    this.logger.info('Cache cleanup complete', { removed, remaining: this.cache.size() });
    return removed;
  }
}
