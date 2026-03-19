import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Address } from '../models/Address';
import { EntityId } from '../../types/common';

/** Repository for address persistence */
export class AddressRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('AddressRepository');
  }

  /** Find all addresses for a user */
  async findByUserId(userId: EntityId): Promise<Address[]> {
    const result = await this.db.query<Address>('SELECT * FROM addresses WHERE user_id = $1', [userId]);
    return result.rows;
  }

  /** Find the default address for a user */
  async findDefault(userId: EntityId): Promise<Address | null> {
    const result = await this.db.query<Address>(
      'SELECT * FROM addresses WHERE user_id = $1 AND is_default = true',
      [userId],
    );
    return result.rows[0] ?? null;
  }

  /** Save a new address */
  async save(address: Address): Promise<Address> {
    await this.db.execute(
      'INSERT INTO addresses (id, user_id, type, line1, city, state, postal_code, country, is_default, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)',
      [address.id, address.userId, address.type, address.line1, address.city, address.state, address.postalCode, address.country, address.isDefault, address.createdAt],
    );
    return address;
  }

  /** Delete an address */
  async delete(addressId: EntityId): Promise<boolean> {
    const affected = await this.db.execute('DELETE FROM addresses WHERE id = $1', [addressId]);
    return affected > 0;
  }
}
