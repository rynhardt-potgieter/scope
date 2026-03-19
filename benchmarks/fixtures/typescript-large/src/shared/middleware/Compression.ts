import { Logger } from '../utils/Logger';

/** Response compression options */
interface CompressionOptions {
  threshold: number;
  level: number;
  mimeTypes: string[];
}

/** Middleware for compressing response bodies */
export class CompressionMiddleware {
  private options: CompressionOptions;
  private logger: Logger;

  constructor(options?: Partial<CompressionOptions>) {
    this.options = {
      threshold: options?.threshold ?? 1024,
      level: options?.level ?? 6,
      mimeTypes: options?.mimeTypes ?? ['application/json', 'text/html', 'text/plain'],
    };
    this.logger = new Logger('CompressionMiddleware');
  }

  /** Check if the response should be compressed based on size and content type */
  shouldCompress(contentLength: number, contentType: string): boolean {
    if (contentLength < this.options.threshold) {
      return false;
    }
    return this.options.mimeTypes.some((type) => contentType.includes(type));
  }

  /** Simulate compressing a response body */
  compress(body: string): { compressed: string; ratio: number } {
    const originalSize = body.length;
    const compressedSize = Math.floor(originalSize * 0.3);
    this.logger.debug('Response compressed', { originalSize, compressedSize });
    return {
      compressed: body.slice(0, compressedSize),
      ratio: compressedSize / originalSize,
    };
  }
}
