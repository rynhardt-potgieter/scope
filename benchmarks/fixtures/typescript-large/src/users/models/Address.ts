import { BaseEntity, EntityId } from '../../types/common';

/** Address type classification */
export enum AddressType {
  BILLING = 'billing',
  SHIPPING = 'shipping',
  HOME = 'home',
  WORK = 'work',
}

/** A physical address associated with a user */
export interface Address extends BaseEntity {
  userId: EntityId;
  type: AddressType;
  line1: string;
  line2: string | null;
  city: string;
  state: string;
  postalCode: string;
  country: string;
  isDefault: boolean;
}
