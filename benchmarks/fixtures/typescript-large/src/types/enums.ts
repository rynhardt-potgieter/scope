/** Roles assignable to users */
export enum UserRole {
  ADMIN = 'admin',
  MANAGER = 'manager',
  EDITOR = 'editor',
  VIEWER = 'viewer',
  API_USER = 'api_user',
}

/** Lifecycle states of a payment */
export enum PaymentStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  COMPLETED = 'completed',
  FAILED = 'failed',
  REFUNDED = 'refunded',
  PARTIALLY_REFUNDED = 'partially_refunded',
  CANCELLED = 'cancelled',
}

/** Lifecycle states of a subscription */
export enum SubscriptionStatus {
  ACTIVE = 'active',
  PAUSED = 'paused',
  PAST_DUE = 'past_due',
  CANCELLED = 'cancelled',
  EXPIRED = 'expired',
  TRIAL = 'trial',
}

/** Lifecycle states of content items */
export enum ContentStatus {
  DRAFT = 'draft',
  IN_REVIEW = 'in_review',
  PUBLISHED = 'published',
  ARCHIVED = 'archived',
  DELETED = 'deleted',
}

/** Channels for delivering notifications */
export enum NotificationChannel {
  EMAIL = 'email',
  PUSH = 'push',
  SMS = 'sms',
  IN_APP = 'in_app',
  WEBHOOK = 'webhook',
}

/** Supported payment processors */
export enum PaymentProcessor {
  STRIPE = 'stripe',
  PAYPAL = 'paypal',
}

/** Subscription billing intervals */
export enum BillingInterval {
  MONTHLY = 'monthly',
  QUARTERLY = 'quarterly',
  YEARLY = 'yearly',
}

/** Invoice states */
export enum InvoiceStatus {
  DRAFT = 'draft',
  ISSUED = 'issued',
  PAID = 'paid',
  OVERDUE = 'overdue',
  VOIDED = 'voided',
}
