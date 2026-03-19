import { BaseEntity, EntityId } from '../../types/common';

/** Extended user profile with display preferences */
export interface UserProfile extends BaseEntity {
  userId: EntityId;
  displayName: string;
  bio: string;
  avatarUrl: string | null;
  timezone: string;
  locale: string;
  phoneNumber: string | null;
  website: string | null;
  socialLinks: SocialLinks;
}

/** Social media links */
export interface SocialLinks {
  twitter: string | null;
  linkedin: string | null;
  github: string | null;
}
