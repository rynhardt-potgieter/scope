import { Logger } from '../utils/Logger';
import { DatabaseClient } from './DatabaseClient';
import { DatabaseConfig } from '../config/DatabaseConfig';

/** Database connection pool manager */
export class ConnectionPool {
  private config: DatabaseConfig;
  private connections: DatabaseClient[];
  private available: DatabaseClient[];
  private logger: Logger;

  constructor(config: DatabaseConfig) {
    this.config = config;
    this.connections = [];
    this.available = [];
    this.logger = new Logger('ConnectionPool');
  }

  /** Initialize the connection pool with the configured number of connections */
  async initialize(): Promise<void> {
    this.logger.info('Initializing connection pool', { maxConnections: this.config.maxConnections });
    for (let i = 0; i < this.config.maxConnections; i++) {
      const client = new DatabaseClient(this.config.getConnectionString());
      await client.connect();
      this.connections.push(client);
      this.available.push(client);
    }
    this.logger.info('Connection pool ready', { size: this.connections.length });
  }

  /** Acquire a connection from the pool */
  async acquire(): Promise<DatabaseClient> {
    if (this.available.length === 0) {
      this.logger.warn('No available connections, waiting');
      await this.sleep(100);
      return this.acquire();
    }
    const client = this.available.pop()!;
    return client;
  }

  /** Release a connection back to the pool */
  release(client: DatabaseClient): void {
    this.available.push(client);
  }

  /** Shut down all connections */
  async shutdown(): Promise<void> {
    this.logger.info('Shutting down connection pool');
    for (const client of this.connections) {
      await client.disconnect();
    }
    this.connections = [];
    this.available = [];
  }

  /** Get pool statistics */
  stats(): { total: number; available: number; inUse: number } {
    return {
      total: this.connections.length,
      available: this.available.length,
      inUse: this.connections.length - this.available.length,
    };
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
