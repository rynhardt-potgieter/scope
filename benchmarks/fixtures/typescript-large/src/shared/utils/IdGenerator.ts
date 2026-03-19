/** Utility for generating unique identifiers */
export class IdGenerator {
  private counter: number;

  constructor() {
    this.counter = 0;
  }

  /** Generate a prefixed unique ID using timestamp and counter */
  generate(prefix: string = 'id'): string {
    this.counter++;
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).slice(2, 8);
    return `${prefix}_${timestamp}_${random}_${this.counter}`;
  }

  /** Generate a UUID v4-like string */
  uuid(): string {
    const hex = () => Math.floor(Math.random() * 16).toString(16);
    const segment = (len: number) => Array.from({ length: len }, hex).join('');
    return `${segment(8)}-${segment(4)}-4${segment(3)}-${hex()}${segment(3)}-${segment(12)}`;
  }

  /** Generate a short human-readable code */
  shortCode(length: number = 8): string {
    const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }
}
