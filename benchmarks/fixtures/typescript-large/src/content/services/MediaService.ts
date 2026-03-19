import { Logger } from '../../shared/utils/Logger';
import { Media, MediaType } from '../models/Media';
import { EntityId } from '../../types/common';
import { ValidationError, NotFoundError } from '../../types/errors';
import { MAX_UPLOAD_SIZE_BYTES, ALLOWED_IMAGE_TYPES } from '../../types/constants';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { StringUtils } from '../../shared/utils/StringUtils';

/** Service for managing media uploads */
export class MediaService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('MediaService');
  }

  /** Upload a new media file */
  async uploadMedia(
    uploadedBy: EntityId,
    originalFilename: string,
    mimeType: string,
    size: number,
    url: string,
  ): Promise<Media> {
    if (size > MAX_UPLOAD_SIZE_BYTES) {
      throw new ValidationError('File too large', { size: `Maximum ${MAX_UPLOAD_SIZE_BYTES} bytes` });
    }

    const type = this.determineType(mimeType);
    const filename = `${Date.now()}_${StringUtils.slugify(originalFilename)}`;

    const media: Media = {
      id: `med_${Date.now()}`,
      uploadedBy,
      filename,
      originalFilename,
      mimeType,
      size,
      url,
      thumbnailUrl: type === MediaType.IMAGE ? `${url}?thumb=true` : null,
      type,
      alt: originalFilename,
      width: null,
      height: null,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    await this.db.execute(
      'INSERT INTO media (id, uploaded_by, filename, mime_type, size, url, type, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)',
      [media.id, media.uploadedBy, media.filename, media.mimeType, media.size, media.url, media.type, media.createdAt],
    );
    this.logger.info('Media uploaded', { mediaId: media.id, type: media.type });
    return media;
  }

  /** Get media by ID */
  async getMedia(mediaId: EntityId): Promise<Media> {
    const result = await this.db.query<Media>('SELECT * FROM media WHERE id = $1', [mediaId]);
    if (result.rows.length === 0) throw new NotFoundError('Media', mediaId);
    return result.rows[0];
  }

  /** Delete a media file */
  async deleteMedia(mediaId: EntityId): Promise<void> {
    await this.db.execute('DELETE FROM media WHERE id = $1', [mediaId]);
    this.logger.info('Media deleted', { mediaId });
  }

  private determineType(mimeType: string): MediaType {
    if (mimeType.startsWith('image/')) return MediaType.IMAGE;
    if (mimeType.startsWith('video/')) return MediaType.VIDEO;
    if (mimeType.startsWith('audio/')) return MediaType.AUDIO;
    return MediaType.DOCUMENT;
  }
}
