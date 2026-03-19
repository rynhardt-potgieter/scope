import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Media, MediaType } from '../models/Media';
import { EntityId } from '../../types/common';

/** Repository for media persistence */
export class MediaRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('MediaRepository');
  }

  /** Find media by ID */
  async findById(mediaId: EntityId): Promise<Media | null> {
    const result = await this.db.query<Media>('SELECT * FROM media WHERE id = $1', [mediaId]);
    return result.rows[0] ?? null;
  }

  /** Find media uploaded by a specific user */
  async findByUploader(userId: EntityId, limit: number = 20): Promise<Media[]> {
    const result = await this.db.query<Media>(
      'SELECT * FROM media WHERE uploaded_by = $1 ORDER BY created_at DESC LIMIT $2',
      [userId, limit],
    );
    return result.rows;
  }

  /** Find media by type */
  async findByType(type: MediaType, limit: number = 20): Promise<Media[]> {
    const result = await this.db.query<Media>(
      'SELECT * FROM media WHERE type = $1 ORDER BY created_at DESC LIMIT $2',
      [type, limit],
    );
    return result.rows;
  }

  /** Save a media record */
  async save(media: Media): Promise<Media> {
    await this.db.execute(
      'INSERT INTO media (id, uploaded_by, filename, mime_type, size, url, type, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)',
      [media.id, media.uploadedBy, media.filename, media.mimeType, media.size, media.url, media.type, media.createdAt],
    );
    return media;
  }

  /** Delete a media record */
  async delete(mediaId: EntityId): Promise<boolean> {
    const affected = await this.db.execute('DELETE FROM media WHERE id = $1', [mediaId]);
    return affected > 0;
  }
}
