import { ValidationError } from '../../types/errors';

/** Validates profile update input */
export class ProfileValidator {
  /** Validate profile update fields */
  validateUpdate(data: { displayName?: string; bio?: string; website?: string }): void {
    const errors: Record<string, string> = {};

    if (data.displayName !== undefined && data.displayName.trim().length === 0) {
      errors.displayName = 'Display name cannot be empty';
    }
    if (data.displayName && data.displayName.length > 50) {
      errors.displayName = 'Display name must be 50 characters or less';
    }

    if (data.bio && data.bio.length > 500) {
      errors.bio = 'Bio must be 500 characters or less';
    }

    if (data.website && !data.website.startsWith('https://')) {
      errors.website = 'Website must start with https://';
    }

    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Profile validation failed', errors);
    }
  }
}
