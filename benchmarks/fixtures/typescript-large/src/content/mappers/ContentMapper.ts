import { Content } from '../models/Content';

/** Content response DTO */
export interface ContentDto {
  id: string;
  title: string;
  slug: string;
  excerpt: string;
  authorId: string;
  categoryId: string | null;
  tags: string[];
  status: string;
  publishedAt: string | null;
  featuredImageUrl: string | null;
  viewCount: number;
  createdAt: string;
  updatedAt: string;
}

/** Content detail DTO (includes body) */
export interface ContentDetailDto extends ContentDto {
  body: string;
}

/** Maps Content entities to DTOs */
export class ContentMapper {
  /** Convert to list DTO (no body) */
  static toDto(content: Content): ContentDto {
    return {
      id: content.id,
      title: content.title,
      slug: content.slug,
      excerpt: content.excerpt,
      authorId: content.authorId,
      categoryId: content.categoryId,
      tags: content.tags,
      status: content.status,
      publishedAt: content.publishedAt?.toISOString() ?? null,
      featuredImageUrl: content.featuredImageUrl,
      viewCount: content.viewCount,
      createdAt: content.createdAt.toISOString(),
      updatedAt: content.updatedAt.toISOString(),
    };
  }

  /** Convert to detail DTO (includes body) */
  static toDetailDto(content: Content): ContentDetailDto {
    return {
      ...ContentMapper.toDto(content),
      body: content.body,
    };
  }

  /** Convert a list to DTOs */
  static toDtoList(contents: Content[]): ContentDto[] {
    return contents.map(ContentMapper.toDto);
  }
}
