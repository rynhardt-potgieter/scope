import { RenderedTemplate } from '../models/Template';

/** Renders email templates with variable substitution */
export class EmailTemplate {
  private templateStore: Map<string, { subject: string; body: string }>;

  constructor() {
    this.templateStore = new Map();
    this.loadDefaults();
  }

  /** Render a template by name with the given variables */
  render(templateName: string, variables: Record<string, string>): RenderedTemplate {
    const template = this.templateStore.get(templateName);
    if (!template) {
      return { subject: 'Notification', body: `Template "${templateName}" not found.` };
    }

    let subject = template.subject;
    let body = template.body;

    for (const [key, value] of Object.entries(variables)) {
      const placeholder = `{{${key}}}`;
      subject = subject.replace(new RegExp(placeholder, 'g'), value);
      body = body.replace(new RegExp(placeholder, 'g'), value);
    }

    return { subject, body };
  }

  /** Register a custom template */
  register(name: string, subject: string, body: string): void {
    this.templateStore.set(name, { subject, body });
  }

  private loadDefaults(): void {
    this.templateStore.set('welcome', {
      subject: 'Welcome to SaaS API, {{name}}!',
      body: 'Hi {{name}}, welcome to our platform. Get started by exploring our features.',
    });
    this.templateStore.set('payment_confirmation', {
      subject: 'Payment Confirmation - {{amount}}',
      body: 'Your payment of {{amount}} has been processed successfully. Transaction ID: {{transactionId}}.',
    });
    this.templateStore.set('password_reset', {
      subject: 'Password Reset Request',
      body: 'Click the following link to reset your password: {{resetLink}}. This link expires in 1 hour.',
    });
    this.templateStore.set('subscription_renewal', {
      subject: 'Subscription Renewed - {{planName}}',
      body: 'Your {{planName}} subscription has been renewed. Next billing date: {{nextBillingDate}}.',
    });
  }
}
