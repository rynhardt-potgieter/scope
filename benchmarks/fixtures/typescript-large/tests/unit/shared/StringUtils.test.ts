import { StringUtils } from '../../../src/shared/utils/StringUtils';

describe('StringUtils', () => {
  describe('slugify', () => {
    it('should convert to lowercase kebab-case', () => {
      expect(StringUtils.slugify('Hello World')).toBe('hello-world');
    });

    it('should remove special characters', () => {
      expect(StringUtils.slugify('Hello! @World#')).toBe('hello-world');
    });

    it('should handle multiple spaces', () => {
      expect(StringUtils.slugify('hello   world')).toBe('hello-world');
    });
  });

  describe('truncate', () => {
    it('should not truncate short strings', () => {
      expect(StringUtils.truncate('short', 10)).toBe('short');
    });

    it('should truncate and add ellipsis', () => {
      expect(StringUtils.truncate('a very long string that needs truncating', 20)).toHaveLength(20);
    });
  });

  describe('sanitize', () => {
    it('should remove HTML tags', () => {
      const result = StringUtils.sanitize('<script>alert("xss")</script>Hello');
      expect(result).not.toContain('<script>');
    });
  });

  describe('capitalize', () => {
    it('should capitalize the first letter', () => {
      expect(StringUtils.capitalize('hello')).toBe('Hello');
    });
  });
});
