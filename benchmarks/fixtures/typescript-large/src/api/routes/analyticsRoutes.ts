/** Route definitions for analytics endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns analytics route definitions */
export function analyticsRoutes(): RouteDefinition[] {
  return [
    { method: 'GET', path: '/analytics/revenue', handler: 'AnalyticsController.getRevenueSummary' },
    { method: 'GET', path: '/analytics/daily-revenue', handler: 'AnalyticsController.getDailyRevenue' },
    { method: 'GET', path: '/health', handler: 'HealthController.check' },
  ];
}
