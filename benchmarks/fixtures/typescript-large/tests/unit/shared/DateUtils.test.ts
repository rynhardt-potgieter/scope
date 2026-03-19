import { DateUtils } from '../../../src/shared/utils/DateUtils';

describe('DateUtils', () => {
  describe('formatDate', () => {
    it('should return ISO 8601 format', () => {
      const date = new Date('2024-01-15T12:00:00Z');
      expect(DateUtils.formatDate(date)).toContain('2024');
    });
  });

  describe('parseDate', () => {
    it('should parse a valid date string', () => {
      const date = DateUtils.parseDate('2024-01-15');
      expect(date).toBeInstanceOf(Date);
    });

    it('should throw on invalid date string', () => {
      expect(() => DateUtils.parseDate('not-a-date')).toThrow();
    });
  });

  describe('isExpired', () => {
    it('should return true for past dates', () => {
      const past = new Date('2020-01-01');
      expect(DateUtils.isExpired(past)).toBe(true);
    });

    it('should return false for future dates', () => {
      const future = new Date('2099-01-01');
      expect(DateUtils.isExpired(future)).toBe(false);
    });
  });

  describe('addDays', () => {
    it('should add the specified number of days', () => {
      const base = new Date('2024-01-01');
      const result = DateUtils.addDays(base, 10);
      expect(DateUtils.diffDays(base, result)).toBe(10);
    });
  });
});
