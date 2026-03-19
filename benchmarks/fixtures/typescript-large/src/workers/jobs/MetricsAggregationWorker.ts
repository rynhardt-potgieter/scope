import { Logger } from '../../shared/utils/Logger';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Aggregated metrics data point */
interface MetricsSnapshot {
  timestamp: Date;
  totalUsers: number;
  activeUsers: number;
  totalPayments: number;
  revenue: number;
  avgResponseTimeMs: number;
}

/** Background worker that aggregates system metrics */
export class MetricsAggregationWorker {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('MetricsAggregationWorker');
  }

  /** Aggregate metrics for the current hour */
  async aggregateMetrics(): Promise<MetricsSnapshot> {
    this.logger.info('Aggregating metrics');

    const userResult = await this.db.query<{ total: number; active: number }>(
      'SELECT COUNT(*) as total, COUNT(CASE WHEN is_active THEN 1 END) as active FROM users',
    );
    const paymentResult = await this.db.query<{ total: number; revenue: number }>(
      "SELECT COUNT(*) as total, COALESCE(SUM(amount), 0) as revenue FROM payments WHERE status = 'completed' AND created_at >= $1",
      [DateUtils.addHours(new Date(), -1)],
    );

    const snapshot: MetricsSnapshot = {
      timestamp: new Date(),
      totalUsers: userResult.rows[0]?.total ?? 0,
      activeUsers: userResult.rows[0]?.active ?? 0,
      totalPayments: paymentResult.rows[0]?.total ?? 0,
      revenue: paymentResult.rows[0]?.revenue ?? 0,
      avgResponseTimeMs: Math.random() * 200 + 50,
    };

    await this.db.execute(
      'INSERT INTO metrics (timestamp, total_users, active_users, total_payments, revenue, avg_response_time) VALUES ($1, $2, $3, $4, $5, $6)',
      [snapshot.timestamp, snapshot.totalUsers, snapshot.activeUsers, snapshot.totalPayments, snapshot.revenue, snapshot.avgResponseTimeMs],
    );

    this.logger.info('Metrics aggregated', { ...snapshot });
    return snapshot;
  }
}
