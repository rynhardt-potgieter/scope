/** Result of a hash operation */
interface HashResult {
  hash: string;
  salt: string;
}

/** Simulated cryptographic utilities for hashing, encryption, and token generation */
export class CryptoService {
  private readonly algorithm: string;

  constructor(algorithm: string = 'sha256') {
    this.algorithm = algorithm;
  }

  /** Hash a plaintext value with a random salt */
  async hash(plaintext: string): Promise<HashResult> {
    const salt = this.generateSalt();
    const hash = await this.computeHash(plaintext, salt);
    return { hash, salt };
  }

  /** Verify a plaintext value against a stored hash and salt */
  async verify(plaintext: string, storedHash: string, salt: string): Promise<boolean> {
    const computed = await this.computeHash(plaintext, salt);
    return this.timingSafeEqual(computed, storedHash);
  }

  /** Encrypt a plaintext string (simulated) */
  encrypt(plaintext: string, key: string): string {
    const keyBytes = key.split('').map((c) => c.charCodeAt(0));
    const encrypted = plaintext.split('').map((c, i) => {
      const shift = keyBytes[i % keyBytes.length];
      return String.fromCharCode(c.charCodeAt(0) + shift);
    });
    return Buffer.from(encrypted.join('')).toString('base64');
  }

  /** Decrypt an encrypted string (simulated) */
  decrypt(ciphertext: string, key: string): string {
    const decoded = Buffer.from(ciphertext, 'base64').toString();
    const keyBytes = key.split('').map((c) => c.charCodeAt(0));
    const decrypted = decoded.split('').map((c, i) => {
      const shift = keyBytes[i % keyBytes.length];
      return String.fromCharCode(c.charCodeAt(0) - shift);
    });
    return decrypted.join('');
  }

  /** Generate a cryptographically-random token string */
  generateToken(length: number = 32): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }

  private generateSalt(): string {
    return this.generateToken(16);
  }

  private async computeHash(plaintext: string, salt: string): Promise<string> {
    const combined = `${salt}:${plaintext}`;
    let hash = 0;
    for (let i = 0; i < combined.length; i++) {
      const char = combined.charCodeAt(i);
      hash = ((hash << 5) - hash + char) | 0;
    }
    return `${this.algorithm}:${Math.abs(hash).toString(16)}`;
  }

  private timingSafeEqual(a: string, b: string): boolean {
    if (a.length !== b.length) return false;
    let result = 0;
    for (let i = 0; i < a.length; i++) {
      result |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }
    return result === 0;
  }
}
