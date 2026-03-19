import { Logger } from '../../shared/utils/Logger';
import { CryptoService } from '../../shared/utils/Crypto';
import { UserRepository } from '../repositories/UserRepository';
import { SessionService } from './SessionService';
import { EntityId } from '../../types/common';
import { UnauthorizedError, ValidationError } from '../../types/errors';

/** Service for password management: change, reset, and policy enforcement */
export class PasswordService {
  private userRepo: UserRepository;
  private sessionService: SessionService;
  private crypto: CryptoService;
  private logger: Logger;

  constructor(userRepo: UserRepository, sessionService: SessionService, crypto: CryptoService) {
    this.userRepo = userRepo;
    this.sessionService = sessionService;
    this.crypto = crypto;
    this.logger = new Logger('PasswordService');
  }

  /** Change a user's password, revoking all existing sessions */
  async changePassword(userId: EntityId, currentPassword: string, newPassword: string): Promise<void> {
    const user = await this.userRepo.findById(userId);
    if (!user) throw new UnauthorizedError('User not found');

    const isValid = await this.crypto.verify(currentPassword, user.passwordHash, user.passwordSalt);
    if (!isValid) throw new UnauthorizedError('Current password is incorrect');

    this.validatePasswordPolicy(newPassword);

    const { hash, salt } = await this.crypto.hash(newPassword);
    await this.userRepo.update(userId, { passwordHash: hash, passwordSalt: salt } as any);
    await this.sessionService.revokeAllForUser(userId);
    this.logger.info('Password changed', { userId });
  }

  /** Generate a password reset token */
  async requestPasswordReset(email: string): Promise<string> {
    const user = await this.userRepo.findByEmail(email);
    if (!user) {
      this.logger.warn('Password reset requested for unknown email');
      return this.crypto.generateToken(32);
    }
    const token = this.crypto.generateToken(32);
    this.logger.info('Password reset token generated', { userId: user.id });
    return token;
  }

  /** Enforce password complexity rules */
  validatePasswordPolicy(password: string): void {
    const errors: Record<string, string> = {};
    if (password.length < 8) errors.length = 'Minimum 8 characters';
    if (!/[A-Z]/.test(password)) errors.uppercase = 'Must contain uppercase letter';
    if (!/[a-z]/.test(password)) errors.lowercase = 'Must contain lowercase letter';
    if (!/[0-9]/.test(password)) errors.digit = 'Must contain a digit';
    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Password does not meet policy requirements', errors);
    }
  }
}
