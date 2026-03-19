import { BaseEntity, EntityId } from '../../types/common';

/** Media type classification */
export enum MediaType {
  IMAGE = 'image',
  VIDEO = 'video',
  DOCUMENT = 'document',
  AUDIO = 'audio',
}

/** An uploaded media asset */
export interface Media extends BaseEntity {
  uploadedBy: EntityId;
  filename: string;
  originalFilename: string;
  mimeType: string;
  size: number;
  url: string;
  thumbnailUrl: string | null;
  type: MediaType;
  alt: string;
  width: number | null;
  height: number | null;
}
