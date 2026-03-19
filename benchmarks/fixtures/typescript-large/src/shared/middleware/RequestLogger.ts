import { Logger } from '../utils/Logger';

/** Captured details of an HTTP request/response pair */
interface RequestLog {
  method: string;
  path: string;
  statusCode: number;
  duration: number;
  userAgent: string;
  ip: string;
}

/** Middleware that logs every HTTP request and response */
export class RequestLogger {
  private logger: Logger;

  constructor() {
    this.logger = new Logger('RequestLogger');
  }

  /** Record the start of a request; returns a function to call when the response is sent */
  start(method: string, path: string, userAgent: string, ip: string): () => void {
    const startTime = Date.now();
    this.logger.debug('Request started', { method, path });

    return () => {
      const duration = Date.now() - startTime;
      const entry: RequestLog = {
        method,
        path,
        statusCode: 200,
        duration,
        userAgent,
        ip,
      };
      if (duration > 1000) {
        this.logger.warn('Slow request', { ...entry });
      } else {
        this.logger.info('Request completed', { ...entry });
      }
    };
  }

  /** Log an error that occurred during request processing */
  logError(method: string, path: string, error: Error): void {
    this.logger.error('Request failed', {
      method,
      path,
      error: error.message,
    });
  }
}
