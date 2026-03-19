import { DatabaseClient } from './DatabaseClient';
import { Logger } from '../utils/Logger';

/** A single database migration with up and down operations */
export interface MigrationStep {
  version: number;
  name: string;
  up: string;
  down: string;
}

/** Status of a migration */
export interface MigrationStatus {
  version: number;
  name: string;
  appliedAt: Date | null;
  pending: boolean;
}

/** Manages database schema migrations */
export class MigrationRunner {
  private db: DatabaseClient;
  private logger: Logger;
  private migrations: MigrationStep[];

  constructor(db: DatabaseClient, migrations: MigrationStep[]) {
    this.db = db;
    this.logger = new Logger('MigrationRunner');
    this.migrations = migrations.sort((a, b) => a.version - b.version);
  }

  /** Apply all pending migrations in order */
  async up(): Promise<number> {
    const statuses = await this.status();
    const pending = statuses.filter((s) => s.pending);
    this.logger.info('Applying migrations', { count: pending.length });

    for (const migration of pending) {
      const step = this.migrations.find((m) => m.version === migration.version);
      if (step) {
        await this.db.execute(step.up);
        await this.db.execute(
          'INSERT INTO migrations (version, name, applied_at) VALUES ($1, $2, $3)',
          [step.version, step.name, new Date()],
        );
        this.logger.info('Applied migration', { version: step.version, name: step.name });
      }
    }
    return pending.length;
  }

  /** Roll back the last applied migration */
  async down(): Promise<void> {
    const statuses = await this.status();
    const applied = statuses.filter((s) => !s.pending).reverse();
    if (applied.length === 0) {
      this.logger.warn('No migrations to roll back');
      return;
    }
    const last = applied[0];
    const step = this.migrations.find((m) => m.version === last.version);
    if (step) {
      await this.db.execute(step.down);
      await this.db.execute('DELETE FROM migrations WHERE version = $1', [step.version]);
      this.logger.info('Rolled back migration', { version: step.version, name: step.name });
    }
  }

  /** Get the status of all known migrations */
  async status(): Promise<MigrationStatus[]> {
    const result = await this.db.query<{ version: number; applied_at: Date }>(
      'SELECT version, applied_at FROM migrations ORDER BY version',
    );
    const appliedVersions = new Set(result.rows.map((r) => r.version));

    return this.migrations.map((m) => ({
      version: m.version,
      name: m.name,
      appliedAt: appliedVersions.has(m.version) ? new Date() : null,
      pending: !appliedVersions.has(m.version),
    }));
  }
}
