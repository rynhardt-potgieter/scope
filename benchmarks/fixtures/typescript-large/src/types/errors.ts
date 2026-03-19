/** Base application error with status code */
export class AppError extends Error {
  public readonly statusCode: number;
  public readonly isOperational: boolean;

  constructor(message: string, statusCode: number, isOperational = true) {
    super(message);
    this.statusCode = statusCode;
    this.isOperational = isOperational;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Validation error for bad input */
export class ValidationError extends AppError {
  public readonly fields: Record<string, string>;

  constructor(message: string, fields: Record<string, string> = {}) {
    super(message, 400);
    this.fields = fields;
  }
}

/** Resource not found */
export class NotFoundError extends AppError {
  public readonly resourceType: string;
  public readonly resourceId: string;

  constructor(resourceType: string, resourceId: string) {
    super(`${resourceType} with id ${resourceId} not found`, 404);
    this.resourceType = resourceType;
    this.resourceId = resourceId;
  }
}

/** Authentication / authorization failure */
export class UnauthorizedError extends AppError {
  constructor(message: string = 'Unauthorized') {
    super(message, 401);
  }
}

/** Forbidden — user lacks permission */
export class ForbiddenError extends AppError {
  constructor(message: string = 'Forbidden') {
    super(message, 403);
  }
}

/** Conflict — duplicate or stale data */
export class ConflictError extends AppError {
  constructor(message: string) {
    super(message, 409);
  }
}
