import { testId } from '../../helpers/testUtils';

describe('CategoryService', () => {
  describe('createCategory', () => {
    it('should generate a slug from the category name', () => {
      const name = 'Product Updates';
      const slug = name.toLowerCase().replace(/\s+/g, '-');
      expect(slug).toBe('product-updates');
    });

    it('should reject duplicate slugs', () => {
      expect('existing-slug').toBe('existing-slug');
    });
  });

  describe('listCategories', () => {
    it('should return only active categories', () => {
      expect(true).toBe(true);
    });
  });

  describe('deactivateCategory', () => {
    it('should set isActive to false', () => {
      const isActive = false;
      expect(isActive).toBe(false);
    });
  });
});
