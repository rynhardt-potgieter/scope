import { Logger } from '../utils/Logger';

/** CORS configuration options */
interface CorsOptions {
  allowedOrigins: string[];
  allowedMethods: string[];
  allowedHeaders: string[];
  maxAge: number;
  credentials: boolean;
}

/** Cross-Origin Resource Sharing middleware */
export class CorsMiddleware {
  private options: CorsOptions;
  private logger: Logger;

  constructor(options?: Partial<CorsOptions>) {
    this.options = {
      allowedOrigins: options?.allowedOrigins ?? ['*'],
      allowedMethods: options?.allowedMethods ?? ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
      allowedHeaders: options?.allowedHeaders ?? ['Content-Type', 'Authorization'],
      maxAge: options?.maxAge ?? 86400,
      credentials: options?.credentials ?? true,
    };
    this.logger = new Logger('CorsMiddleware');
  }

  /** Check whether a given origin is allowed */
  isOriginAllowed(origin: string): boolean {
    if (this.options.allowedOrigins.includes('*')) {
      return true;
    }
    return this.options.allowedOrigins.includes(origin);
  }

  /** Build CORS headers for a given origin */
  getHeaders(origin: string): Record<string, string> {
    const headers: Record<string, string> = {};
    if (this.isOriginAllowed(origin)) {
      headers['Access-Control-Allow-Origin'] = origin;
      headers['Access-Control-Allow-Methods'] = this.options.allowedMethods.join(', ');
      headers['Access-Control-Allow-Headers'] = this.options.allowedHeaders.join(', ');
      headers['Access-Control-Max-Age'] = String(this.options.maxAge);
      if (this.options.credentials) {
        headers['Access-Control-Allow-Credentials'] = 'true';
      }
    } else {
      this.logger.warn('Origin not allowed', { origin });
    }
    return headers;
  }
}
