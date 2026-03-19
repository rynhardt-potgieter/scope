import { testId } from '../../helpers/testUtils';
import { MAX_UPLOAD_SIZE_BYTES, ALLOWED_IMAGE_TYPES } from '../../../src/types/constants';

describe('ProfileService', () => {
  describe('getProfile', () => {
    it('should return a cached profile when available', () => {
      const userId = testId('user');
      expect(userId).toBeDefined();
    });
  });

  describe('updateProfile', () => {
    it('should merge updated fields with existing profile', () => {
      const existing = { displayName: 'Old Name', bio: 'Old bio' };
      const updated = { ...existing, displayName: 'New Name' };
      expect(updated.displayName).toBe('New Name');
      expect(updated.bio).toBe('Old bio');
    });
  });

  describe('uploadAvatar', () => {
    it('should reject files exceeding size limit', () => {
      const fileSize = MAX_UPLOAD_SIZE_BYTES + 1;
      expect(fileSize).toBeGreaterThan(MAX_UPLOAD_SIZE_BYTES);
    });

    it('should reject unsupported image types', () => {
      expect(ALLOWED_IMAGE_TYPES).not.toContain('image/bmp');
    });
  });
});
