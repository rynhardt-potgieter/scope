/** Request to update a profile */
export interface UpdateProfileRequest {
  displayName?: string;
  bio?: string;
  timezone?: string;
  socialLinks?: {
    twitter?: string;
    linkedin?: string;
    github?: string;
  };
}

/** Profile response */
export interface ProfileResponse {
  userId: string;
  displayName: string;
  bio: string;
  avatarUrl: string | null;
  timezone: string;
  locale: string;
}

/** Avatar upload response */
export interface AvatarUploadResponse {
  avatarUrl: string;
}
