import { SortOrder } from '../../types/common';

/** Fluent SQL query builder */
export class QueryBuilder {
  private tableName: string;
  private selectColumns: string[];
  private whereClauses: string[];
  private whereParams: unknown[];
  private joinClauses: string[];
  private orderByClause: string | null;
  private limitValue: number | null;
  private offsetValue: number | null;

  constructor(table: string) {
    this.tableName = table;
    this.selectColumns = [];
    this.whereClauses = [];
    this.whereParams = [];
    this.joinClauses = [];
    this.orderByClause = null;
    this.limitValue = null;
    this.offsetValue = null;
  }

  /** Specify columns to select */
  select(...columns: string[]): QueryBuilder {
    this.selectColumns.push(...columns);
    return this;
  }

  /** Add a WHERE condition */
  where(condition: string, ...params: unknown[]): QueryBuilder {
    this.whereClauses.push(condition);
    this.whereParams.push(...params);
    return this;
  }

  /** Add a JOIN clause */
  join(table: string, on: string): QueryBuilder {
    this.joinClauses.push(`JOIN ${table} ON ${on}`);
    return this;
  }

  /** Add a LEFT JOIN clause */
  leftJoin(table: string, on: string): QueryBuilder {
    this.joinClauses.push(`LEFT JOIN ${table} ON ${on}`);
    return this;
  }

  /** Set ORDER BY */
  orderBy(column: string, order: SortOrder = SortOrder.ASC): QueryBuilder {
    this.orderByClause = `${column} ${order}`;
    return this;
  }

  /** Set LIMIT */
  limit(count: number): QueryBuilder {
    this.limitValue = count;
    return this;
  }

  /** Set OFFSET */
  offset(count: number): QueryBuilder {
    this.offsetValue = count;
    return this;
  }

  /** Build the final SQL string and parameter array */
  build(): { sql: string; params: unknown[] } {
    const cols = this.selectColumns.length > 0 ? this.selectColumns.join(', ') : '*';
    let sql = `SELECT ${cols} FROM ${this.tableName}`;

    for (const join of this.joinClauses) {
      sql += ` ${join}`;
    }
    if (this.whereClauses.length > 0) {
      sql += ` WHERE ${this.whereClauses.join(' AND ')}`;
    }
    if (this.orderByClause) {
      sql += ` ORDER BY ${this.orderByClause}`;
    }
    if (this.limitValue !== null) {
      sql += ` LIMIT ${this.limitValue}`;
    }
    if (this.offsetValue !== null) {
      sql += ` OFFSET ${this.offsetValue}`;
    }
    return { sql, params: this.whereParams };
  }
}
