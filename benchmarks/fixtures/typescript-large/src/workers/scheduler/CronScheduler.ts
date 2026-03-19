import { Logger } from '../../shared/utils/Logger';

/** A scheduled job definition */
interface ScheduledJob {
  name: string;
  cronExpression: string;
  handler: () => Promise<void>;
  enabled: boolean;
  lastRun: Date | null;
  nextRun: Date | null;
}

/** Simple cron-like scheduler for background jobs */
export class CronScheduler {
  private jobs: Map<string, ScheduledJob>;
  private logger: Logger;
  private running: boolean;

  constructor() {
    this.jobs = new Map();
    this.logger = new Logger('CronScheduler');
    this.running = false;
  }

  /** Register a job with a cron expression */
  register(name: string, cronExpression: string, handler: () => Promise<void>): void {
    this.jobs.set(name, {
      name,
      cronExpression,
      handler,
      enabled: true,
      lastRun: null,
      nextRun: null,
    });
    this.logger.info('Job registered', { name, cronExpression });
  }

  /** Start the scheduler */
  async start(): Promise<void> {
    this.running = true;
    this.logger.info('Scheduler started', { jobCount: this.jobs.size });
    while (this.running) {
      for (const [, job] of this.jobs) {
        if (job.enabled && this.shouldRun(job)) {
          try {
            await job.handler();
            job.lastRun = new Date();
            this.logger.info('Job executed', { name: job.name });
          } catch (error) {
            this.logger.error('Job failed', { name: job.name, error: String(error) });
          }
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 60000));
    }
  }

  /** Stop the scheduler */
  stop(): void {
    this.running = false;
    this.logger.info('Scheduler stopped');
  }

  /** Enable or disable a specific job */
  setEnabled(name: string, enabled: boolean): void {
    const job = this.jobs.get(name);
    if (job) {
      job.enabled = enabled;
      this.logger.info('Job toggled', { name, enabled });
    }
  }

  private shouldRun(job: ScheduledJob): boolean {
    if (!job.lastRun) return true;
    return Date.now() - job.lastRun.getTime() > 60000;
  }
}
