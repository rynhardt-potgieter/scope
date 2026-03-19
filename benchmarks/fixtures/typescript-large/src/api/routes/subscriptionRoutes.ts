/** Route definitions for subscription endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns subscription route definitions */
export function subscriptionRoutes(): RouteDefinition[] {
  return [
    { method: 'POST', path: '/subscriptions', handler: 'SubscriptionController.create' },
    { method: 'POST', path: '/subscriptions/:id/renew', handler: 'SubscriptionController.renewSubscription' },
    { method: 'POST', path: '/subscriptions/:id/cancel', handler: 'SubscriptionController.cancel' },
    { method: 'GET', path: '/subscriptions/:id', handler: 'SubscriptionController.getSubscription' },
  ];
}
