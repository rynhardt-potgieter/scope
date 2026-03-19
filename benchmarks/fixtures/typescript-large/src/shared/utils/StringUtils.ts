/** String manipulation utilities */
export class StringUtils {
  /** Convert a string to a URL-safe slug */
  static slugify(text: string): string {
    return text
      .toLowerCase()
      .trim()
      .replace(/[^\w\s-]/g, '')
      .replace(/[\s_]+/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '');
  }

  /** Truncate a string to a maximum length, adding ellipsis if needed */
  static truncate(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength - 3) + '...';
  }

  /** Remove HTML tags and dangerous characters from a string */
  static sanitize(text: string): string {
    return text
      .replace(/<[^>]*>/g, '')
      .replace(/[<>&"']/g, (char) => {
        const entities: Record<string, string> = {
          '<': '&lt;',
          '>': '&gt;',
          '&': '&amp;',
          '"': '&quot;',
          "'": '&#x27;',
        };
        return entities[char] ?? char;
      });
  }

  /** Capitalize the first letter of a string */
  static capitalize(text: string): string {
    if (text.length === 0) return text;
    return text.charAt(0).toUpperCase() + text.slice(1);
  }

  /** Convert a camelCase or PascalCase string to kebab-case */
  static toKebabCase(text: string): string {
    return text.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
  }

  /** Generate a random alphanumeric string of the given length */
  static randomString(length: number): string {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }
}
