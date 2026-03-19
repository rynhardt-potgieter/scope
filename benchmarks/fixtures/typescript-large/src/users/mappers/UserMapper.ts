import { UserInfo } from '../services/UserService';

/** User list item DTO */
export interface UserListItemDto {
  id: string;
  email: string;
  fullName: string;
  role: string;
  isActive: boolean;
  createdAt: string;
}

/** Maps UserInfo to DTOs */
export class UserMapper {
  /** Convert UserInfo to a list item DTO */
  static toListItem(user: UserInfo): UserListItemDto {
    return {
      id: user.id,
      email: user.email,
      fullName: `${user.firstName} ${user.lastName}`,
      role: user.role,
      isActive: user.isActive,
      createdAt: user.createdAt.toISOString(),
    };
  }

  /** Convert a list of UserInfo to DTOs */
  static toListItems(users: UserInfo[]): UserListItemDto[] {
    return users.map(UserMapper.toListItem);
  }
}
