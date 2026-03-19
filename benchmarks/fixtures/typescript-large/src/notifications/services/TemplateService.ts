import { Logger } from '../../shared/utils/Logger';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Template, RenderedTemplate } from '../models/Template';
import { NotificationChannel } from '../../types/enums';
import { NotFoundError } from '../../types/errors';
import { EntityId } from '../../types/common';

/** Service for managing and rendering notification templates */
export class TemplateService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('TemplateService');
  }

  /** Get a template by name and channel */
  async getTemplate(name: string, channel: NotificationChannel): Promise<Template> {
    const result = await this.db.query<Template>(
      'SELECT * FROM notification_templates WHERE name = $1 AND channel = $2',
      [name, channel],
    );
    if (result.rows.length === 0) {
      throw new NotFoundError('Template', `${name}:${channel}`);
    }
    return result.rows[0];
  }

  /** Render a template with variable substitution */
  async render(name: string, channel: NotificationChannel, variables: Record<string, string>): Promise<RenderedTemplate> {
    const template = await this.getTemplate(name, channel);
    let subject = template.subject;
    let body = template.bodyTemplate;

    for (const [key, value] of Object.entries(variables)) {
      const placeholder = `{{${key}}}`;
      subject = subject.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
      body = body.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
    }

    this.logger.debug('Template rendered', { name, channel });
    return { subject, body };
  }

  /** List all templates */
  async listTemplates(): Promise<Template[]> {
    const result = await this.db.query<Template>('SELECT * FROM notification_templates WHERE is_active = true ORDER BY name');
    return result.rows;
  }

  /** Create a new template */
  async createTemplate(name: string, channel: NotificationChannel, subject: string, bodyTemplate: string, variables: string[]): Promise<Template> {
    const template: Template = {
      id: `tmpl_${Date.now()}`,
      name,
      channel,
      subject,
      bodyTemplate,
      variables,
      isActive: true,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    await this.db.execute(
      'INSERT INTO notification_templates (id, name, channel, subject, body_template, variables, is_active, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)',
      [template.id, name, channel, subject, bodyTemplate, JSON.stringify(variables), true, template.createdAt],
    );
    this.logger.info('Template created', { name, channel });
    return template;
  }
}
