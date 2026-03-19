import { JWT_EXPIRATION_SECONDS, REFRESH_TOKEN_EXPIRATION_SECONDS } from '../../types/constants';

/** Authentication provider configuration */
export interface AuthConfigValues {
  jwtSecret: string;
  jwtExpirationSeconds: number;
  refreshTokenExpirationSeconds: number;
  issuer: string;
  audience: string;
  bcryptRounds: number;
}

/** Loads authentication configuration from environment variables */
export class AuthConfig {
  private values: AuthConfigValues;

  constructor() {
    this.values = this.load();
  }

  /** Get the JWT secret key */
  get jwtSecret(): string {
    return this.values.jwtSecret;
  }

  /** Get JWT token expiration in seconds */
  get jwtExpirationSeconds(): number {
    return this.values.jwtExpirationSeconds;
  }

  /** Get refresh token expiration in seconds */
  get refreshTokenExpirationSeconds(): number {
    return this.values.refreshTokenExpirationSeconds;
  }

  /** Get the JWT issuer claim */
  get issuer(): string {
    return this.values.issuer;
  }

  /** Get the JWT audience claim */
  get audience(): string {
    return this.values.audience;
  }

  /** Get bcrypt salt rounds */
  get bcryptRounds(): number {
    return this.values.bcryptRounds;
  }

  private load(): AuthConfigValues {
    return {
      jwtSecret: process.env.JWT_SECRET ?? 'default-dev-secret-change-in-production',
      jwtExpirationSeconds: parseInt(
        process.env.JWT_EXPIRATION ?? String(JWT_EXPIRATION_SECONDS),
        10,
      ),
      refreshTokenExpirationSeconds: parseInt(
        process.env.REFRESH_TOKEN_EXPIRATION ?? String(REFRESH_TOKEN_EXPIRATION_SECONDS),
        10,
      ),
      issuer: process.env.JWT_ISSUER ?? 'saas-api',
      audience: process.env.JWT_AUDIENCE ?? 'saas-api-clients',
      bcryptRounds: parseInt(process.env.BCRYPT_ROUNDS ?? '12', 10),
    };
  }
}
