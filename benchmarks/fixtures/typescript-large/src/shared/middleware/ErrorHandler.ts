import { Logger } from '../utils/Logger';
import { AppError } from '../../types/errors';
import { ApiResponse } from '../../types/common';
import { isAppError } from '../../types/guards';

/** HTTP request abstraction */
interface Request {
  method: string;
  path: string;
  headers: Record<string, string>;
}

/** HTTP response abstraction */
interface Response {
  status(code: number): Response;
  json(body: unknown): void;
}

/** Central error handler that catches errors and formats API responses */
export class ErrorHandler {
  private logger: Logger;

  constructor() {
    this.logger = new Logger('ErrorHandler');
  }

  /** Handle an error and send the appropriate response */
  handle(error: unknown, req: Request, res: Response): void {
    if (isAppError(error)) {
      this.handleAppError(error, req, res);
    } else if (error instanceof Error) {
      this.handleUnknownError(error, req, res);
    } else {
      this.handleUnknownError(new Error(String(error)), req, res);
    }
  }

  private handleAppError(error: AppError, req: Request, res: Response): void {
    this.logger.warn('Application error', {
      method: req.method,
      path: req.path,
      statusCode: error.statusCode,
      message: error.message,
    });

    const response: ApiResponse<null> = {
      success: false,
      data: null,
      message: error.message,
      timestamp: new Date(),
    };
    res.status(error.statusCode).json(response);
  }

  private handleUnknownError(error: Error, req: Request, res: Response): void {
    this.logger.error('Unhandled error', {
      method: req.method,
      path: req.path,
      error: error.message,
      stack: error.stack,
    });

    const response: ApiResponse<null> = {
      success: false,
      data: null,
      message: 'Internal server error',
      timestamp: new Date(),
    };
    res.status(500).json(response);
  }
}
