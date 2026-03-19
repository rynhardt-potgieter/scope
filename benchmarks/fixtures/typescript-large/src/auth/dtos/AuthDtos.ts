/** Login request payload */
export interface LoginRequest {
  email: string;
  password: string;
}

/** Login response with tokens */
export interface LoginResponse {
  userId: string;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  accessToken: string;
  refreshToken: string;
  expiresAt: string;
}

/** Token pair for refresh operations */
export interface TokenPair {
  accessToken: string;
  refreshToken: string;
  expiresAt: string;
}

/** Registration request payload */
export interface RegisterRequest {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
}

/** Token refresh request */
export interface RefreshTokenRequest {
  refreshToken: string;
}
