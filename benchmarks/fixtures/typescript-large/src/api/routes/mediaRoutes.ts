/** Route definitions for media endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns media route definitions */
export function mediaRoutes(): RouteDefinition[] {
  return [
    { method: 'POST', path: '/media/upload', handler: 'MediaController.upload' },
    { method: 'GET', path: '/media/:id', handler: 'MediaController.getMedia' },
    { method: 'DELETE', path: '/media/:id', handler: 'MediaController.deleteMedia' },
  ];
}
