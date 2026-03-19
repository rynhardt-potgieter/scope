import { Logger } from '../../shared/utils/Logger';
import { Content } from '../models/Content';
import { EntityId } from '../../types/common';
import { ContentStatus } from '../../types/enums';

/** Search result with relevance score */
export interface SearchResult {
  contentId: EntityId;
  title: string;
  excerpt: string;
  score: number;
}

/** Full-text search service for content */
export class SearchService {
  private index: Map<string, { contentId: EntityId; title: string; excerpt: string; terms: string[] }>;
  private logger: Logger;

  constructor() {
    this.index = new Map();
    this.logger = new Logger('SearchService');
  }

  /** Index a content item for search */
  async indexContent(content: Content): Promise<void> {
    const terms = this.tokenize(`${content.title} ${content.body} ${content.tags.join(' ')}`);
    this.index.set(content.id, {
      contentId: content.id,
      title: content.title,
      excerpt: content.excerpt,
      terms,
    });
    this.logger.debug('Content indexed', { contentId: content.id, termCount: terms.length });
  }

  /** Remove a content item from the search index */
  async removeContent(contentId: EntityId): Promise<void> {
    this.index.delete(contentId);
    this.logger.debug('Content removed from index', { contentId });
  }

  /** Search for content matching a query string */
  async search(query: string, limit: number = 20): Promise<SearchResult[]> {
    const queryTerms = this.tokenize(query);
    const results: SearchResult[] = [];

    for (const [, entry] of this.index) {
      const score = this.calculateScore(queryTerms, entry.terms);
      if (score > 0) {
        results.push({
          contentId: entry.contentId,
          title: entry.title,
          excerpt: entry.excerpt,
          score,
        });
      }
    }

    results.sort((a, b) => b.score - a.score);
    this.logger.debug('Search executed', { query, resultCount: results.length });
    return results.slice(0, limit);
  }

  private tokenize(text: string): string[] {
    return text
      .toLowerCase()
      .replace(/[^\w\s]/g, '')
      .split(/\s+/)
      .filter((t) => t.length > 2);
  }

  private calculateScore(queryTerms: string[], docTerms: string[]): number {
    let matches = 0;
    for (const qt of queryTerms) {
      for (const dt of docTerms) {
        if (dt.includes(qt)) {
          matches++;
        }
      }
    }
    return matches / Math.max(1, queryTerms.length);
  }
}
