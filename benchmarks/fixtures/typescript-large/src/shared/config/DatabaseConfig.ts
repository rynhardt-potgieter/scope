/** Database connection configuration */
export interface DatabaseConfigValues {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  maxConnections: number;
  idleTimeoutMs: number;
  ssl: boolean;
}

/** Loads database configuration from environment variables */
export class DatabaseConfig {
  private values: DatabaseConfigValues;

  constructor() {
    this.values = this.load();
  }

  /** Build a connection string from the config values */
  getConnectionString(): string {
    const proto = this.values.ssl ? 'postgresql+ssl' : 'postgresql';
    return `${proto}://${this.values.username}:***@${this.values.host}:${this.values.port}/${this.values.database}`;
  }

  /** Get the maximum number of connections in the pool */
  get maxConnections(): number {
    return this.values.maxConnections;
  }

  /** Get idle connection timeout in milliseconds */
  get idleTimeoutMs(): number {
    return this.values.idleTimeoutMs;
  }

  /** Get the raw config values */
  getValues(): DatabaseConfigValues {
    return { ...this.values };
  }

  private load(): DatabaseConfigValues {
    return {
      host: process.env.DB_HOST ?? 'localhost',
      port: parseInt(process.env.DB_PORT ?? '5432', 10),
      database: process.env.DB_NAME ?? 'saas_api',
      username: process.env.DB_USER ?? 'postgres',
      password: process.env.DB_PASSWORD ?? '',
      maxConnections: parseInt(process.env.DB_MAX_CONNECTIONS ?? '20', 10),
      idleTimeoutMs: parseInt(process.env.DB_IDLE_TIMEOUT ?? '30000', 10),
      ssl: process.env.DB_SSL === 'true',
    };
  }
}
