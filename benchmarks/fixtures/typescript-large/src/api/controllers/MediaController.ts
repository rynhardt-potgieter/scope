import { Logger } from '../../shared/utils/Logger';
import { MediaService } from '../../content/services/MediaService';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { Media } from '../../content/models/Media';

/** Controller for media upload endpoints */
export class MediaController {
  private mediaService: MediaService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(mediaService: MediaService, authGuard: AuthGuard) {
    this.mediaService = mediaService;
    this.authGuard = authGuard;
    this.logger = new Logger('MediaController');
  }

  /** POST /media/upload */
  async upload(
    authHeader: string,
    originalFilename: string,
    mimeType: string,
    size: number,
    url: string,
  ): Promise<ApiResponse<{ mediaId: string; url: string }>> {
    const user = this.authGuard.guard(authHeader);
    const media = await this.mediaService.uploadMedia(user.sub, originalFilename, mimeType, size, url);
    this.logger.info('Media uploaded via API', { mediaId: media.id });
    return {
      success: true,
      data: { mediaId: media.id, url: media.url },
      message: 'Media uploaded',
      timestamp: new Date(),
    };
  }

  /** GET /media/:id */
  async getMedia(authHeader: string, mediaId: string): Promise<ApiResponse<Media>> {
    this.authGuard.guard(authHeader);
    const media = await this.mediaService.getMedia(mediaId);
    return {
      success: true,
      data: media,
      message: 'Media retrieved',
      timestamp: new Date(),
    };
  }

  /** DELETE /media/:id */
  async deleteMedia(authHeader: string, mediaId: string): Promise<ApiResponse<{ deleted: boolean }>> {
    this.authGuard.guard(authHeader);
    await this.mediaService.deleteMedia(mediaId);
    return {
      success: true,
      data: { deleted: true },
      message: 'Media deleted',
      timestamp: new Date(),
    };
  }
}
