import { AuthController } from '../controllers/AuthController';

/** Route definitions for authentication endpoints */
export interface RouteDefinition {
  method: string;
  path: string;
  handler: string;
}

/** Returns auth route definitions */
export function authRoutes(): RouteDefinition[] {
  return [
    { method: 'POST', path: '/auth/login', handler: 'AuthController.login' },
    { method: 'POST', path: '/auth/register', handler: 'AuthController.register' },
    { method: 'POST', path: '/auth/refresh', handler: 'AuthController.refresh' },
    { method: 'POST', path: '/auth/logout', handler: 'AuthController.logout' },
  ];
}
