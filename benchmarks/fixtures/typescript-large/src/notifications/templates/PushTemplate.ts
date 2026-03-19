/** Push notification template with title and body */
export class PushTemplate {
  private templates: Map<string, { title: string; body: string }>;

  constructor() {
    this.templates = new Map();
    this.loadDefaults();
  }

  /** Render a push notification template */
  render(templateName: string, variables: Record<string, string>): { title: string; body: string } {
    const template = this.templates.get(templateName);
    if (!template) {
      return { title: 'Notification', body: 'You have a new notification.' };
    }

    let title = template.title;
    let body = template.body;

    for (const [key, value] of Object.entries(variables)) {
      title = title.replace(`{{${key}}}`, value);
      body = body.replace(`{{${key}}}`, value);
    }

    return { title, body };
  }

  /** Register a custom push template */
  register(name: string, title: string, body: string): void {
    this.templates.set(name, { title, body });
  }

  private loadDefaults(): void {
    this.templates.set('new_message', {
      title: 'New Message',
      body: '{{senderName}} sent you a message.',
    });
    this.templates.set('payment_received', {
      title: 'Payment Received',
      body: 'You received a payment of {{amount}}.',
    });
    this.templates.set('subscription_expiring', {
      title: 'Subscription Expiring',
      body: 'Your {{planName}} subscription expires in {{daysLeft}} days.',
    });
  }
}
