import { Logger } from './Logger';
import { RETRY_DELAY_BASE_MS, MAX_RETRY_ATTEMPTS } from '../../types/constants';

/** Options for retry behavior */
export interface RetryOptions {
  maxAttempts: number;
  baseDelayMs: number;
  backoffMultiplier: number;
}

/** Utility for retrying failed async operations with exponential backoff */
export class RetryHelper {
  private options: RetryOptions;
  private logger: Logger;

  constructor(options?: Partial<RetryOptions>) {
    this.options = {
      maxAttempts: options?.maxAttempts ?? MAX_RETRY_ATTEMPTS,
      baseDelayMs: options?.baseDelayMs ?? RETRY_DELAY_BASE_MS,
      backoffMultiplier: options?.backoffMultiplier ?? 2,
    };
    this.logger = new Logger('RetryHelper');
  }

  /** Execute an async function with retry logic */
  async execute<T>(operation: () => Promise<T>, operationName: string): Promise<T> {
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= this.options.maxAttempts; attempt++) {
      try {
        const result = await operation();
        if (attempt > 1) {
          this.logger.info('Retry succeeded', { operationName, attempt });
        }
        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        this.logger.warn('Operation failed, retrying', {
          operationName,
          attempt,
          maxAttempts: this.options.maxAttempts,
          error: lastError.message,
        });

        if (attempt < this.options.maxAttempts) {
          const delay = this.options.baseDelayMs * Math.pow(this.options.backoffMultiplier, attempt - 1);
          await this.sleep(delay);
        }
      }
    }

    this.logger.error('All retries exhausted', { operationName, maxAttempts: this.options.maxAttempts });
    throw lastError;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
