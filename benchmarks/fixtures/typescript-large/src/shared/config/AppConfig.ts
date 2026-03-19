/** Application environment */
export enum Environment {
  DEVELOPMENT = 'development',
  STAGING = 'staging',
  PRODUCTION = 'production',
  TEST = 'test',
}

/** Top-level application configuration */
export interface AppConfigValues {
  env: Environment;
  port: number;
  host: string;
  apiPrefix: string;
  logLevel: string;
  corsOrigins: string[];
}

/** Loads and validates application configuration from environment variables */
export class AppConfig {
  private values: AppConfigValues;

  constructor() {
    this.values = this.load();
  }

  /** Get the current environment */
  get env(): Environment {
    return this.values.env;
  }

  /** Get the server port */
  get port(): number {
    return this.values.port;
  }

  /** Get the server host */
  get host(): string {
    return this.values.host;
  }

  /** Get the API prefix (e.g., /api/v1) */
  get apiPrefix(): string {
    return this.values.apiPrefix;
  }

  /** Check if running in production */
  isProduction(): boolean {
    return this.values.env === Environment.PRODUCTION;
  }

  /** Check if running in test mode */
  isTest(): boolean {
    return this.values.env === Environment.TEST;
  }

  private load(): AppConfigValues {
    return {
      env: (process.env.NODE_ENV as Environment) ?? Environment.DEVELOPMENT,
      port: parseInt(process.env.PORT ?? '3000', 10),
      host: process.env.HOST ?? '0.0.0.0',
      apiPrefix: process.env.API_PREFIX ?? '/api/v1',
      logLevel: process.env.LOG_LEVEL ?? 'info',
      corsOrigins: (process.env.CORS_ORIGINS ?? '*').split(','),
    };
  }
}
