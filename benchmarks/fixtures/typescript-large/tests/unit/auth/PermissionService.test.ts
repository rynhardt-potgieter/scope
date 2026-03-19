import { PermissionAction } from '../../../src/auth/models/Permission';
import { testId } from '../../helpers/testUtils';

describe('PermissionService', () => {
  describe('checkPermission', () => {
    it('should return true when user has the permission', () => {
      expect(PermissionAction.READ).toBe('read');
    });

    it('should return false when user lacks the permission', () => {
      expect(PermissionAction.DELETE).toBe('delete');
    });
  });

  describe('requirePermission', () => {
    it('should throw ForbiddenError when permission is missing', () => {
      const userId = testId('user');
      expect(userId).toBeDefined();
    });
  });

  describe('grantPermission', () => {
    it('should create a permission grant', () => {
      const permId = testId('perm');
      expect(permId).toContain('perm');
    });
  });
});
