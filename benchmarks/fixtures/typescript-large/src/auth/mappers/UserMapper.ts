import { User } from '../models/User';

/** User data transfer object (safe for API responses) */
export interface UserDto {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  isActive: boolean;
  lastLoginAt: string | null;
  createdAt: string;
}

/** Maps User entities to DTOs and vice versa */
export class UserMapper {
  /** Convert a User entity to a safe DTO (no password fields) */
  static toDto(user: User): UserDto {
    return {
      id: user.id,
      email: user.email,
      firstName: user.firstName,
      lastName: user.lastName,
      role: user.role,
      isActive: user.isActive,
      lastLoginAt: user.lastLoginAt?.toISOString() ?? null,
      createdAt: user.createdAt.toISOString(),
    };
  }

  /** Convert a list of User entities to DTOs */
  static toDtoList(users: User[]): UserDto[] {
    return users.map(UserMapper.toDto);
  }
}
