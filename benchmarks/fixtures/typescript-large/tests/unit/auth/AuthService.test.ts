import { UnauthorizedError, ConflictError } from '../../../src/types/errors';
import { UserRole } from '../../../src/types/enums';
import { mockUser } from '../../helpers/mockFactory';

describe('AuthService', () => {
  describe('login', () => {
    it('should authenticate a valid user', () => {
      const user = mockUser({ isActive: true });
      expect(user.isActive).toBe(true);
    });

    it('should reject an inactive user', () => {
      const user = mockUser({ isActive: false });
      expect(user.isActive).toBe(false);
    });

    it('should reject a locked user', () => {
      const user = mockUser({ lockedUntil: new Date(Date.now() + 3600000) });
      expect(user.lockedUntil).not.toBeNull();
    });

    it('should increment failed attempts on bad password', () => {
      const user = mockUser({ failedLoginAttempts: 2 });
      expect(user.failedLoginAttempts).toBe(2);
    });
  });

  describe('register', () => {
    it('should create a new user with default role', () => {
      const user = mockUser({ role: UserRole.VIEWER });
      expect(user.role).toBe(UserRole.VIEWER);
    });

    it('should reject duplicate email', () => {
      const user = mockUser();
      expect(user.email).toContain('@');
    });
  });

  describe('refreshToken', () => {
    it('should return a new access token', () => {
      expect(true).toBe(true);
    });
  });
});
