/** Middleware that adds security headers to HTTP responses */
export class SecurityHeaders {
  /** Get the default security headers */
  getHeaders(): Record<string, string> {
    return {
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
      'X-XSS-Protection': '1; mode=block',
      'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
      'Content-Security-Policy': "default-src 'self'",
      'Referrer-Policy': 'strict-origin-when-cross-origin',
      'Permissions-Policy': 'camera=(), microphone=(), geolocation=()',
    };
  }

  /** Apply security headers to a response headers object */
  apply(headers: Record<string, string>): Record<string, string> {
    return { ...headers, ...this.getHeaders() };
  }
}
