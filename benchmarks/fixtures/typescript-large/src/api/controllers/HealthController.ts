import { Logger } from '../../shared/utils/Logger';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { CacheService } from '../../shared/utils/Cache';
import { ApiResponse } from '../../types/common';

/** Health check result */
interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  uptime: number;
  checks: {
    database: boolean;
    cache: boolean;
  };
  version: string;
}

/** Controller for health check endpoints */
export class HealthController {
  private db: DatabaseClient;
  private cache: CacheService;
  private logger: Logger;
  private startTime: Date;

  constructor(db: DatabaseClient, cache: CacheService) {
    this.db = db;
    this.cache = cache;
    this.logger = new Logger('HealthController');
    this.startTime = new Date();
  }

  /** GET /health */
  async check(): Promise<ApiResponse<HealthStatus>> {
    const dbHealthy = await this.checkDatabase();
    const cacheHealthy = this.checkCache();
    const allHealthy = dbHealthy && cacheHealthy;

    const status: HealthStatus = {
      status: allHealthy ? 'healthy' : 'degraded',
      uptime: Date.now() - this.startTime.getTime(),
      checks: {
        database: dbHealthy,
        cache: cacheHealthy,
      },
      version: '1.0.0',
    };

    return {
      success: true,
      data: status,
      message: status.status,
      timestamp: new Date(),
    };
  }

  private async checkDatabase(): Promise<boolean> {
    try {
      await this.db.query('SELECT 1');
      return true;
    } catch {
      this.logger.error('Database health check failed');
      return false;
    }
  }

  private checkCache(): boolean {
    try {
      this.cache.set('health_check', true, 5000);
      return this.cache.get<boolean>('health_check') === true;
    } catch {
      this.logger.error('Cache health check failed');
      return false;
    }
  }
}
