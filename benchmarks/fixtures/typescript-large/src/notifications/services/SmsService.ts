import { Logger } from '../../shared/utils/Logger';
import { EntityId } from '../../types/common';

/** SMS delivery result */
export interface SmsResult {
  messageId: string;
  delivered: boolean;
}

/** Service for sending SMS messages */
export class SmsService {
  private provider: string;
  private logger: Logger;

  constructor(provider: string = 'twilio') {
    this.provider = provider;
    this.logger = new Logger('SmsService');
  }

  /** Send an SMS to a user's registered phone number */
  async sendSms(userId: EntityId, message: string): Promise<SmsResult> {
    this.logger.info('Sending SMS', { userId, provider: this.provider });
    if (message.length > 160) {
      this.logger.warn('SMS message exceeds 160 characters', { length: message.length });
    }
    const messageId = `sms_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    return { messageId, delivered: true };
  }

  /** Send a verification code via SMS */
  async sendVerificationCode(userId: EntityId, code: string): Promise<SmsResult> {
    const message = `Your verification code is: ${code}. It expires in 10 minutes.`;
    return this.sendSms(userId, message);
  }
}
