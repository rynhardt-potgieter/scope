import { ValidationError } from '../../types/errors';
import { isUuid, isEmail } from '../../types/guards';

/** Generic request validation middleware */
export class RequestValidator {
  /** Validate that required fields are present in a request body */
  validateRequired(body: Record<string, unknown>, requiredFields: string[]): void {
    const missing = requiredFields.filter((field) => body[field] === undefined || body[field] === null);
    if (missing.length > 0) {
      const errors: Record<string, string> = {};
      for (const field of missing) {
        errors[field] = `${field} is required`;
      }
      throw new ValidationError('Missing required fields', errors);
    }
  }

  /** Validate that an ID parameter is a valid UUID */
  validateId(id: string, fieldName: string = 'id'): void {
    if (!id || id.trim().length === 0) {
      throw new ValidationError(`${fieldName} is required`, { [fieldName]: 'Required' });
    }
  }

  /** Validate pagination parameters */
  validatePagination(page: unknown, pageSize: unknown): { page: number; pageSize: number } {
    const pageNum = typeof page === 'number' ? page : parseInt(String(page ?? '1'), 10);
    const sizeNum = typeof pageSize === 'number' ? pageSize : parseInt(String(pageSize ?? '20'), 10);

    if (isNaN(pageNum) || pageNum < 1) {
      throw new ValidationError('Invalid page number', { page: 'Must be a positive integer' });
    }
    if (isNaN(sizeNum) || sizeNum < 1 || sizeNum > 100) {
      throw new ValidationError('Invalid page size', { pageSize: 'Must be between 1 and 100' });
    }

    return { page: pageNum, pageSize: sizeNum };
  }
}
