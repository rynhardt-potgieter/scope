import { ValidationError } from '../../types/errors';

/** Generic validation utility with chainable rules */
export class Validator {
  private errors: Record<string, string>;
  private fieldName: string;

  constructor() {
    this.errors = {};
    this.fieldName = '';
  }

  /** Start validating a field */
  field(name: string): Validator {
    this.fieldName = name;
    return this;
  }

  /** Require the field to be a non-empty string */
  required(value: unknown): Validator {
    if (value === undefined || value === null || (typeof value === 'string' && value.trim().length === 0)) {
      this.errors[this.fieldName] = `${this.fieldName} is required`;
    }
    return this;
  }

  /** Require the field to be at most maxLength characters */
  maxLength(value: string, max: number): Validator {
    if (value && value.length > max) {
      this.errors[this.fieldName] = `${this.fieldName} must be at most ${max} characters`;
    }
    return this;
  }

  /** Require the field to be at least minLength characters */
  minLength(value: string, min: number): Validator {
    if (value && value.length < min) {
      this.errors[this.fieldName] = `${this.fieldName} must be at least ${min} characters`;
    }
    return this;
  }

  /** Require the field to be a positive number */
  positive(value: number): Validator {
    if (typeof value === 'number' && value <= 0) {
      this.errors[this.fieldName] = `${this.fieldName} must be positive`;
    }
    return this;
  }

  /** Throw ValidationError if any errors accumulated */
  validate(): void {
    if (Object.keys(this.errors).length > 0) {
      throw new ValidationError('Validation failed', this.errors);
    }
  }
}
