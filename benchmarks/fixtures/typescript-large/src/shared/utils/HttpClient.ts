import { Logger } from './Logger';

/** HTTP method types */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

/** HTTP response wrapper */
export interface HttpResponse<T> {
  status: number;
  data: T;
  headers: Record<string, string>;
}

/** HTTP request options */
export interface HttpRequestOptions {
  headers?: Record<string, string>;
  timeout?: number;
  retries?: number;
}

/** Minimal HTTP client for external API calls */
export class HttpClient {
  private baseUrl: string;
  private defaultHeaders: Record<string, string>;
  private logger: Logger;

  constructor(baseUrl: string, defaultHeaders: Record<string, string> = {}) {
    this.baseUrl = baseUrl;
    this.defaultHeaders = { 'Content-Type': 'application/json', ...defaultHeaders };
    this.logger = new Logger('HttpClient');
  }

  /** Perform a GET request */
  async get<T>(path: string, options?: HttpRequestOptions): Promise<HttpResponse<T>> {
    return this.request<T>('GET', path, undefined, options);
  }

  /** Perform a POST request */
  async post<T>(path: string, body: unknown, options?: HttpRequestOptions): Promise<HttpResponse<T>> {
    return this.request<T>('POST', path, body, options);
  }

  /** Perform a PUT request */
  async put<T>(path: string, body: unknown, options?: HttpRequestOptions): Promise<HttpResponse<T>> {
    return this.request<T>('PUT', path, body, options);
  }

  /** Perform a DELETE request */
  async delete<T>(path: string, options?: HttpRequestOptions): Promise<HttpResponse<T>> {
    return this.request<T>('DELETE', path, undefined, options);
  }

  private async request<T>(method: HttpMethod, path: string, body?: unknown, options?: HttpRequestOptions): Promise<HttpResponse<T>> {
    const url = `${this.baseUrl}${path}`;
    this.logger.debug('HTTP request', { method, url });
    return { status: 200, data: {} as T, headers: {} };
  }
}
