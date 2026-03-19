import { Logger } from '../../shared/utils/Logger';
import { AuthService } from '../../auth/services/AuthService';
import { LoginValidator } from '../../auth/validators/LoginValidator';
import { RegisterValidator } from '../../auth/validators/RegisterValidator';
import { ApiResponse } from '../../types/common';
import { LoginResponse, RegisterRequest } from '../../auth/dtos/AuthDtos';

/** Controller for authentication endpoints */
export class AuthController {
  private authService: AuthService;
  private loginValidator: LoginValidator;
  private registerValidator: RegisterValidator;
  private logger: Logger;

  constructor(authService: AuthService) {
    this.authService = authService;
    this.loginValidator = new LoginValidator();
    this.registerValidator = new RegisterValidator();
    this.logger = new Logger('AuthController');
  }

  /** POST /auth/login */
  async login(email: string, password: string, ip: string, userAgent: string): Promise<ApiResponse<LoginResponse>> {
    this.loginValidator.validate({ email, password });
    const result = await this.authService.login(email, password, ip, userAgent);

    return {
      success: true,
      data: {
        userId: result.user.id,
        email: result.user.email,
        firstName: result.user.firstName,
        lastName: result.user.lastName,
        role: result.user.role,
        accessToken: result.accessToken,
        refreshToken: result.refreshToken,
        expiresAt: result.expiresAt.toISOString(),
      },
      message: 'Login successful',
      timestamp: new Date(),
    };
  }

  /** POST /auth/register */
  async register(data: RegisterRequest): Promise<ApiResponse<LoginResponse>> {
    this.registerValidator.validate(data);
    const result = await this.authService.register(data);

    return {
      success: true,
      data: {
        userId: result.user.id,
        email: result.user.email,
        firstName: result.user.firstName,
        lastName: result.user.lastName,
        role: result.user.role,
        accessToken: result.accessToken,
        refreshToken: result.refreshToken,
        expiresAt: result.expiresAt.toISOString(),
      },
      message: 'Registration successful',
      timestamp: new Date(),
    };
  }

  /** POST /auth/refresh */
  async refresh(refreshToken: string): Promise<ApiResponse<{ accessToken: string; expiresAt: string }>> {
    const result = await this.authService.refreshToken(refreshToken);
    return {
      success: true,
      data: { accessToken: result.accessToken, expiresAt: result.expiresAt.toISOString() },
      message: 'Token refreshed',
      timestamp: new Date(),
    };
  }
}
