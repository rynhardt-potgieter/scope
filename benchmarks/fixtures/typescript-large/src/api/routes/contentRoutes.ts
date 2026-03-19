/** Route definitions for content endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns content route definitions */
export function contentRoutes(): RouteDefinition[] {
  return [
    { method: 'POST', path: '/content', handler: 'ContentController.create' },
    { method: 'PUT', path: '/content/:id', handler: 'ContentController.update' },
    { method: 'DELETE', path: '/content/:id', handler: 'ContentController.delete' },
    { method: 'GET', path: '/content/search', handler: 'ContentController.search' },
    { method: 'GET', path: '/content/:id', handler: 'ContentController.getContent' },
  ];
}
