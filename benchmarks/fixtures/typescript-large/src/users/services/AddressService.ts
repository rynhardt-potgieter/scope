import { Logger } from '../../shared/utils/Logger';
import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Address, AddressType } from '../models/Address';
import { EntityId } from '../../types/common';
import { NotFoundError, ValidationError } from '../../types/errors';

/** Service for managing user addresses */
export class AddressService {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('AddressService');
  }

  /** Get all addresses for a user */
  async getAddresses(userId: EntityId): Promise<Address[]> {
    const result = await this.db.query<Address>('SELECT * FROM addresses WHERE user_id = $1 ORDER BY is_default DESC', [userId]);
    return result.rows;
  }

  /** Add a new address for a user */
  async addAddress(userId: EntityId, data: Omit<Address, 'id' | 'userId' | 'createdAt' | 'updatedAt'>): Promise<Address> {
    const now = new Date();
    const address: Address = {
      id: `addr_${Date.now()}`,
      userId,
      ...data,
      createdAt: now,
      updatedAt: now,
    };
    await this.db.execute(
      'INSERT INTO addresses (id, user_id, type, line1, line2, city, state, postal_code, country, is_default, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)',
      [address.id, userId, address.type, address.line1, address.line2, address.city, address.state, address.postalCode, address.country, address.isDefault, now],
    );
    if (address.isDefault) {
      await this.db.execute('UPDATE addresses SET is_default = false WHERE user_id = $1 AND id != $2', [userId, address.id]);
    }
    this.logger.info('Address added', { userId, addressId: address.id });
    return address;
  }

  /** Delete an address */
  async deleteAddress(addressId: EntityId, userId: EntityId): Promise<void> {
    const affected = await this.db.execute('DELETE FROM addresses WHERE id = $1 AND user_id = $2', [addressId, userId]);
    if (affected === 0) throw new NotFoundError('Address', addressId);
    this.logger.info('Address deleted', { addressId });
  }

  /** Set an address as the default */
  async setDefault(addressId: EntityId, userId: EntityId): Promise<void> {
    await this.db.execute('UPDATE addresses SET is_default = false WHERE user_id = $1', [userId]);
    await this.db.execute('UPDATE addresses SET is_default = true WHERE id = $1 AND user_id = $2', [addressId, userId]);
    this.logger.info('Default address set', { addressId, userId });
  }
}
