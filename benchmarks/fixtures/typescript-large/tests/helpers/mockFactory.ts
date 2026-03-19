import { User } from '../../src/auth/models/User';
import { Session } from '../../src/auth/models/Session';
import { Payment } from '../../src/payments/models/Payment';
import { Invoice, InvoiceLineItem } from '../../src/payments/models/Invoice';
import { Subscription } from '../../src/payments/models/Subscription';
import { Content } from '../../src/content/models/Content';
import { Notification, DeliveryStatus } from '../../src/notifications/models/Notification';
import { UserRole, PaymentStatus, SubscriptionStatus, BillingInterval, InvoiceStatus, ContentStatus, NotificationChannel } from '../../src/types/enums';
import { Currency, createMoney } from '../../src/types/money';
import { testId } from './testUtils';

/** Create a mock User entity */
export function mockUser(overrides?: Partial<User>): User {
  const now = new Date();
  return {
    id: testId('user'),
    email: `test-${Date.now()}@example.com`,
    passwordHash: 'hashed',
    passwordSalt: 'salt',
    firstName: 'Test',
    lastName: 'User',
    role: UserRole.VIEWER,
    isActive: true,
    lastLoginAt: null,
    failedLoginAttempts: 0,
    lockedUntil: null,
    emailVerified: true,
    verificationToken: null,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/** Create a mock Payment entity */
export function mockPayment(overrides?: Partial<Payment>): Payment {
  const now = new Date();
  return {
    id: testId('pay'),
    userId: testId('user'),
    amount: createMoney(49.99, Currency.USD),
    status: PaymentStatus.COMPLETED,
    processor: 'stripe',
    processorTransactionId: testId('ch'),
    description: 'Test payment',
    metadata: {},
    failureReason: null,
    refundedAmount: null,
    completedAt: now,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/** Create a mock Subscription entity */
export function mockSubscription(overrides?: Partial<Subscription>): Subscription {
  const now = new Date();
  return {
    id: testId('sub'),
    userId: testId('user'),
    planName: 'Pro',
    status: SubscriptionStatus.ACTIVE,
    billingInterval: BillingInterval.MONTHLY,
    amount: createMoney(29.99, Currency.USD),
    currentPeriodStart: now,
    currentPeriodEnd: new Date(now.getTime() + 30 * 86400000),
    cancelledAt: null,
    cancelReason: null,
    trialEndsAt: null,
    nextBillingDate: new Date(now.getTime() + 30 * 86400000),
    failedPaymentAttempts: 0,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/** Create a mock Content entity */
export function mockContent(overrides?: Partial<Content>): Content {
  const now = new Date();
  return {
    id: testId('cnt'),
    title: 'Test Article',
    slug: 'test-article',
    body: 'This is a test article body with enough content to be realistic.',
    excerpt: 'This is a test article...',
    authorId: testId('user'),
    categoryId: null,
    tags: ['test', 'fixture'],
    status: ContentStatus.DRAFT,
    publishedAt: null,
    featuredImageUrl: null,
    viewCount: 0,
    metadata: {},
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/** Create a mock Notification entity */
export function mockNotification(overrides?: Partial<Notification>): Notification {
  const now = new Date();
  return {
    id: testId('ntf'),
    userId: testId('user'),
    channel: NotificationChannel.EMAIL,
    subject: 'Test Notification',
    body: 'This is a test notification.',
    status: DeliveryStatus.PENDING,
    sentAt: null,
    deliveredAt: null,
    failureReason: null,
    retryCount: 0,
    metadata: {},
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}
