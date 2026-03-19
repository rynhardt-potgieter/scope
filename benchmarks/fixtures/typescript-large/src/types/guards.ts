import { AppError, ValidationError, NotFoundError, UnauthorizedError } from './errors';
import { Money } from './money';
import { ApiResponse } from './common';

/** Type guard: checks if a value is a non-null object */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** Type guard: checks if an error is an AppError */
export function isAppError(error: unknown): error is AppError {
  return error instanceof AppError;
}

/** Type guard: checks if an error is a ValidationError */
export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError;
}

/** Type guard: checks if an error is a NotFoundError */
export function isNotFoundError(error: unknown): error is NotFoundError {
  return error instanceof NotFoundError;
}

/** Type guard: checks if an error is an UnauthorizedError */
export function isUnauthorizedError(error: unknown): error is UnauthorizedError {
  return error instanceof UnauthorizedError;
}

/** Type guard: checks if a value is a valid Money object */
export function isMoney(value: unknown): value is Money {
  return isObject(value) && typeof value.amount === 'number' && typeof value.currency === 'string';
}

/** Type guard: checks if a value is a successful ApiResponse */
export function isSuccessResponse<T>(response: ApiResponse<T>): boolean {
  return response.success === true && response.data !== undefined;
}

/** Type guard: checks if a string is a valid UUID v4 */
export function isUuid(value: string): boolean {
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(value);
}

/** Type guard: checks if a string is a valid email address */
export function isEmail(value: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(value);
}
