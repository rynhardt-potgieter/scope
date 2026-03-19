/** Date formatting and manipulation utilities */
export class DateUtils {
  /** Format a Date as ISO 8601 string */
  static formatDate(date: Date): string {
    return date.toISOString();
  }

  /** Parse a date string into a Date object; throws on invalid input */
  static parseDate(dateString: string): Date {
    const parsed = new Date(dateString);
    if (isNaN(parsed.getTime())) {
      throw new Error(`Invalid date string: ${dateString}`);
    }
    return parsed;
  }

  /** Check whether a date is in the past */
  static isExpired(date: Date): boolean {
    return date.getTime() < Date.now();
  }

  /** Add a number of days to a date, returning a new Date */
  static addDays(date: Date, days: number): Date {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return result;
  }

  /** Add a number of hours to a date */
  static addHours(date: Date, hours: number): Date {
    const result = new Date(date);
    result.setTime(result.getTime() + hours * 3600_000);
    return result;
  }

  /** Calculate the difference in days between two dates */
  static diffDays(a: Date, b: Date): number {
    const msPerDay = 86_400_000;
    return Math.floor(Math.abs(a.getTime() - b.getTime()) / msPerDay);
  }

  /** Returns the start of day (midnight) for a given date */
  static startOfDay(date: Date): Date {
    const result = new Date(date);
    result.setHours(0, 0, 0, 0);
    return result;
  }

  /** Returns the end of day (23:59:59.999) for a given date */
  static endOfDay(date: Date): Date {
    const result = new Date(date);
    result.setHours(23, 59, 59, 999);
    return result;
  }
}
