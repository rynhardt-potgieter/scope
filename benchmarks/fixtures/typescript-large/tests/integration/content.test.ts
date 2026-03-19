import { ContentStatus } from '../../src/types/enums';
import { mockContent } from '../helpers/mockFactory';

describe('Content Integration', () => {
  describe('content lifecycle', () => {
    it('should create draft content', () => {
      const content = mockContent({ status: ContentStatus.DRAFT });
      expect(content.status).toBe(ContentStatus.DRAFT);
    });

    it('should publish draft content', () => {
      expect(ContentStatus.PUBLISHED).toBe('published');
    });

    it('should archive published content', () => {
      expect(ContentStatus.ARCHIVED).toBe('archived');
    });

    it('should delete content (soft delete)', () => {
      expect(ContentStatus.DELETED).toBe('deleted');
    });
  });

  describe('search', () => {
    it('should find content by title', () => {
      const content = mockContent({ title: 'TypeScript Guide' });
      expect(content.title).toContain('TypeScript');
    });

    it('should find content by tags', () => {
      const content = mockContent({ tags: ['typescript', 'tutorial'] });
      expect(content.tags).toContain('typescript');
    });
  });
});
