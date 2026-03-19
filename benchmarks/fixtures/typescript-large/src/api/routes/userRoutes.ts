/** Route definitions for user endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns user route definitions */
export function userRoutes(): RouteDefinition[] {
  return [
    { method: 'GET', path: '/users/:id', handler: 'UserController.getUser' },
    { method: 'PUT', path: '/users/:id', handler: 'UserController.updateUser' },
    { method: 'DELETE', path: '/users/:id', handler: 'UserController.deleteUser' },
    { method: 'GET', path: '/users', handler: 'UserController.searchUsers' },
  ];
}
