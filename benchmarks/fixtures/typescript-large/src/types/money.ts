/** Supported currencies */
export enum Currency {
  USD = 'USD',
  EUR = 'EUR',
  GBP = 'GBP',
  ZAR = 'ZAR',
  JPY = 'JPY',
}

/** Value object representing a monetary amount */
export interface Money {
  amount: number;
  currency: Currency;
}

/** Creates a Money value, ensuring amount is non-negative */
export function createMoney(amount: number, currency: Currency): Money {
  if (amount < 0) {
    throw new Error(`Money amount cannot be negative: ${amount}`);
  }
  return { amount: Math.round(amount * 100) / 100, currency };
}

/** Adds two Money values; throws if currencies differ */
export function addMoney(a: Money, b: Money): Money {
  if (a.currency !== b.currency) {
    throw new Error(`Cannot add ${a.currency} and ${b.currency}`);
  }
  return createMoney(a.amount + b.amount, a.currency);
}

/** Subtracts b from a; throws if currencies differ or result would be negative */
export function subtractMoney(a: Money, b: Money): Money {
  if (a.currency !== b.currency) {
    throw new Error(`Cannot subtract ${a.currency} and ${b.currency}`);
  }
  if (a.amount < b.amount) {
    throw new Error(`Insufficient amount: ${a.amount} < ${b.amount}`);
  }
  return createMoney(a.amount - b.amount, a.currency);
}

/** Formats a Money value as a human-readable string */
export function formatMoney(money: Money): string {
  return `${money.currency} ${money.amount.toFixed(2)}`;
}
