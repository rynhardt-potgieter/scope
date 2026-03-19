import { ValidationError } from '../../types/errors';
import { isEmail } from '../../types/guards';
import { CreateUserData } from '../models/User';

/** Validates registration request data */
export class RegisterValidator {
  /** Validate registration input */
  validate(input: Partial<CreateUserData>): void {
    const errors: Record<string, string> = {};

    if (!input.email || input.email.trim().length === 0) {
      errors.email = 'Email is required';
    } else if (!isEmail(input.email)) {
      errors.email = 'Invalid email format';
    }

    if (!input.password || input.password.length === 0) {
      errors.password = 'Password is required';
    } else if (input.password.length < 8) {
      errors.password = 'Password must be at least 8 characters';
    } else if (!/[A-Z]/.test(input.password)) {
      errors.password = 'Password must contain at least one uppercase letter';
    } else if (!/[0-9]/.test(input.password)) {
      errors.password = 'Password must contain at least one digit';
    }

    if (!input.firstName || input.firstName.trim().length === 0) {
      errors.firstName = 'First name is required';
    } else if (input.firstName.length > 100) {
      errors.firstName = 'First name must be 100 characters or less';
    }

    if (!input.lastName || input.lastName.trim().length === 0) {
      errors.lastName = 'Last name is required';
    } else if (input.lastName.length > 100) {
      errors.lastName = 'Last name must be 100 characters or less';
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Registration validation failed', errors);
    }
  }
}
