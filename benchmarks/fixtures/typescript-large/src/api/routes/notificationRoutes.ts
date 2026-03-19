/** Route definitions for notification endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns notification route definitions */
export function notificationRoutes(): RouteDefinition[] {
  return [
    { method: 'GET', path: '/notifications', handler: 'NotificationController.getNotifications' },
    { method: 'PUT', path: '/notifications/:id/read', handler: 'NotificationController.markRead' },
  ];
}
