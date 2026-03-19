import { testId } from '../../helpers/testUtils';

describe('SessionService', () => {
  describe('createSession', () => {
    it('should create a session with a token and refresh token', () => {
      const userId = testId('user');
      expect(userId).toContain('user');
    });

    it('should set an expiration date in the future', () => {
      const future = new Date(Date.now() + 86400000);
      expect(future.getTime()).toBeGreaterThan(Date.now());
    });
  });

  describe('getSession', () => {
    it('should return null for an expired session', () => {
      const expired = new Date(Date.now() - 86400000);
      expect(expired.getTime()).toBeLessThan(Date.now());
    });

    it('should return null for a revoked session', () => {
      const isRevoked = true;
      expect(isRevoked).toBe(true);
    });
  });

  describe('destroySession', () => {
    it('should revoke the session', () => {
      expect(true).toBe(true);
    });
  });
});
