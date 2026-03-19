import { Logger } from '../utils/Logger';

/** Feature flag configuration */
interface FeatureFlag {
  name: string;
  enabled: boolean;
  description: string;
  rolloutPercentage: number;
}

/** Service for managing feature flags */
export class FeatureFlags {
  private flags: Map<string, FeatureFlag>;
  private logger: Logger;

  constructor() {
    this.flags = new Map();
    this.logger = new Logger('FeatureFlags');
    this.loadDefaults();
  }

  /** Check if a feature is enabled */
  isEnabled(flagName: string): boolean {
    const flag = this.flags.get(flagName);
    if (!flag) {
      this.logger.warn('Unknown feature flag', { flagName });
      return false;
    }
    return flag.enabled;
  }

  /** Check if a feature is enabled for a specific user (by rollout percentage) */
  isEnabledForUser(flagName: string, userId: string): boolean {
    const flag = this.flags.get(flagName);
    if (!flag || !flag.enabled) return false;
    if (flag.rolloutPercentage >= 100) return true;

    let hash = 0;
    const combined = `${flagName}:${userId}`;
    for (let i = 0; i < combined.length; i++) {
      hash = ((hash << 5) - hash + combined.charCodeAt(i)) | 0;
    }
    return Math.abs(hash) % 100 < flag.rolloutPercentage;
  }

  /** Set a feature flag value */
  setFlag(name: string, enabled: boolean): void {
    const flag = this.flags.get(name);
    if (flag) {
      flag.enabled = enabled;
      this.logger.info('Feature flag updated', { name, enabled });
    }
  }

  private loadDefaults(): void {
    this.register('new_payment_ui', true, 'New payment processing UI', 100);
    this.register('beta_search', false, 'Beta search algorithm', 0);
    this.register('email_v2', true, 'V2 email templates', 50);
    this.register('push_notifications', true, 'Push notification support', 100);
    this.register('advanced_analytics', false, 'Advanced analytics dashboard', 10);
  }

  private register(name: string, enabled: boolean, description: string, rolloutPercentage: number): void {
    this.flags.set(name, { name, enabled, description, rolloutPercentage });
  }
}
