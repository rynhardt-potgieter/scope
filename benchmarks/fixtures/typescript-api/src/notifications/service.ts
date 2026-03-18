/** Service for sending notifications to users via different channels. */
export class NotificationService {
  private emailEndpoint: string;
  private smsEndpoint: string;

  constructor(emailEndpoint: string, smsEndpoint: string) {
    this.emailEndpoint = emailEndpoint;
    this.smsEndpoint = smsEndpoint;
  }

  /** Send an email notification to the given address. */
  sendEmail(to: string, subject: string, body: string): boolean {
    // Simulate sending email
    console.log(`[Email -> ${to}] ${subject}: ${body}`);
    return true;
  }

  /** Send an SMS notification to the given phone number. */
  sendSms(phoneNumber: string, message: string): boolean {
    // Simulate sending SMS
    console.log(`[SMS -> ${phoneNumber}] ${message}`);
    return true;
  }
}
