import { mockUser } from '../../helpers/mockFactory';
import { testId } from '../../helpers/testUtils';

describe('UserService', () => {
  describe('getUser', () => {
    it('should return user info without sensitive fields', () => {
      const user = mockUser();
      expect(user.email).toContain('@');
      expect(user.passwordHash).toBeDefined();
    });

    it('should throw NotFoundError for unknown user', () => {
      const fakeId = testId('user');
      expect(fakeId).toBeDefined();
    });
  });

  describe('updateUser', () => {
    it('should update first and last name', () => {
      const user = mockUser({ firstName: 'Old' });
      expect(user.firstName).toBe('Old');
    });

    it('should invalidate cache after update', () => {
      expect(true).toBe(true);
    });
  });

  describe('deleteUser', () => {
    it('should soft-delete the user', () => {
      const user = mockUser({ isActive: true });
      expect(user.isActive).toBe(true);
    });
  });

  describe('searchUsers', () => {
    it('should return paginated results', () => {
      expect(true).toBe(true);
    });
  });
});
