import { ValidationError } from '../../types/errors';
import { MAX_TITLE_LENGTH, MAX_BODY_LENGTH } from '../../types/constants';

/** Validates content creation and update input */
export class ContentValidator {
  /** Validate content creation fields */
  validateCreate(data: { title: string; body: string }): void {
    const errors: Record<string, string> = {};

    if (!data.title || data.title.trim().length === 0) {
      errors.title = 'Title is required';
    } else if (data.title.length > MAX_TITLE_LENGTH) {
      errors.title = `Title must be ${MAX_TITLE_LENGTH} characters or less`;
    }

    if (!data.body || data.body.trim().length === 0) {
      errors.body = 'Body is required';
    } else if (data.body.length > MAX_BODY_LENGTH) {
      errors.body = `Body must be ${MAX_BODY_LENGTH} characters or less`;
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Content validation failed', errors);
    }
  }

  /** Validate content update fields */
  validateUpdate(data: { title?: string; body?: string }): void {
    const errors: Record<string, string> = {};

    if (data.title !== undefined && data.title.trim().length === 0) {
      errors.title = 'Title cannot be empty';
    }
    if (data.title && data.title.length > MAX_TITLE_LENGTH) {
      errors.title = `Title must be ${MAX_TITLE_LENGTH} characters or less`;
    }
    if (data.body && data.body.length > MAX_BODY_LENGTH) {
      errors.body = `Body must be ${MAX_BODY_LENGTH} characters or less`;
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Content validation failed', errors);
    }
  }
}
