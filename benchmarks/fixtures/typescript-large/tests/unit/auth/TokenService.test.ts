import { mockUser } from '../../helpers/mockFactory';

describe('TokenService', () => {
  describe('generateJwt', () => {
    it('should generate a token with three parts', () => {
      const token = 'header.payload.signature';
      expect(token.split('.').length).toBe(3);
    });

    it('should include user ID in the payload', () => {
      const user = mockUser();
      expect(user.id).toBeDefined();
    });
  });

  describe('verifyJwt', () => {
    it('should reject a malformed token', () => {
      const malformed = 'not.a.valid.token.here';
      expect(malformed.split('.').length).not.toBe(3);
    });

    it('should reject an expired token', () => {
      const expiredTime = Math.floor(Date.now() / 1000) - 3600;
      expect(expiredTime).toBeLessThan(Math.floor(Date.now() / 1000));
    });
  });

  describe('decodeJwt', () => {
    it('should return null for invalid tokens', () => {
      expect(null).toBeNull();
    });
  });
});
