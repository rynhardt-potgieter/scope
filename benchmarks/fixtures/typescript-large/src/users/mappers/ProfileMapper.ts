import { UserProfile } from '../models/UserProfile';

/** Profile response DTO */
export interface ProfileDto {
  userId: string;
  displayName: string;
  bio: string;
  avatarUrl: string | null;
  timezone: string;
  locale: string;
  socialLinks: {
    twitter: string | null;
    linkedin: string | null;
    github: string | null;
  };
}

/** Maps UserProfile entities to DTOs */
export class ProfileMapper {
  /** Convert a UserProfile entity to a response DTO */
  static toDto(profile: UserProfile): ProfileDto {
    return {
      userId: profile.userId,
      displayName: profile.displayName,
      bio: profile.bio,
      avatarUrl: profile.avatarUrl,
      timezone: profile.timezone,
      locale: profile.locale,
      socialLinks: {
        twitter: profile.socialLinks.twitter,
        linkedin: profile.socialLinks.linkedin,
        github: profile.socialLinks.github,
      },
    };
  }
}
