import { ValidationError } from '../../types/errors';
import { isEmail } from '../../types/guards';

/** Input for login validation */
export interface LoginInput {
  email: string;
  password: string;
}

/** Validates login request data */
export class LoginValidator {
  /** Validate login input and throw ValidationError if invalid */
  validate(input: LoginInput): void {
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
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Login validation failed', errors);
    }
  }
}
