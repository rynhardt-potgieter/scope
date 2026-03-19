import { ValidationError } from '../../types/errors';
import { isEmail } from '../../types/guards';

/** Validates user update input */
export class UserValidator {
  /** Validate user update fields */
  validateUpdate(data: { firstName?: string; lastName?: string; email?: string }): void {
    const errors: Record<string, string> = {};

    if (data.firstName !== undefined && data.firstName.trim().length === 0) {
      errors.firstName = 'First name cannot be empty';
    }
    if (data.firstName && data.firstName.length > 100) {
      errors.firstName = 'First name must be 100 characters or less';
    }

    if (data.lastName !== undefined && data.lastName.trim().length === 0) {
      errors.lastName = 'Last name cannot be empty';
    }
    if (data.lastName && data.lastName.length > 100) {
      errors.lastName = 'Last name must be 100 characters or less';
    }

    if (data.email !== undefined && !isEmail(data.email)) {
      errors.email = 'Invalid email format';
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('User validation failed', errors);
    }
  }
}
