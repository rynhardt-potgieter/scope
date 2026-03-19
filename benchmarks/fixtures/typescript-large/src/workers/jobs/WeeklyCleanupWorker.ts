import { Logger } from '../../shared/utils/Logger';
import { SessionService } from '../../auth/services/SessionService';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Background worker for weekly maintenance tasks */
export class WeeklyCleanupWorker {
  private sessionService: SessionService;
  private db: DatabaseClient;
  private logger: Logger;

  constructor(sessionService: SessionService, db: DatabaseClient) {
    this.sessionService = sessionService;
    this.db = db;
    this.logger = new Logger('WeeklyCleanupWorker');
  }

  /** Clean up expired sessions */
  async cleanupExpiredSessions(): Promise<number> {
    this.logger.info('Cleaning up expired sessions');
    const count = await this.sessionService.cleanupExpired();
    this.logger.info('Expired sessions cleaned up', { count });
    return count;
  }

  /** Clean up old log entries */
  async cleanupOldLogs(): Promise<number> {
    const cutoff = DateUtils.addDays(new Date(), -90);
    this.logger.info('Cleaning up old logs', { cutoff: DateUtils.formatDate(cutoff) });
    const deleted = await this.db.execute('DELETE FROM logs WHERE created_at < $1', [cutoff]);
    this.logger.info('Old logs cleaned up', { deleted });
    return deleted;
  }

  /** Run all weekly cleanup tasks */
  async runAll(): Promise<void> {
    this.logger.info('Starting weekly cleanup');
    const sessions = await this.cleanupExpiredSessions();
    const logs = await this.cleanupOldLogs();
    this.logger.info('Weekly cleanup complete', { sessions, logs });
  }
}
