/** Represents a user in the system. */
export interface User {
  id: string;
  email: string;
  name: string;
  createdAt: Date;
}

/** Data access layer for user records. */
export class UserRepository {
  private users: Map<string, User>;

  constructor() {
    this.users = new Map();
  }

  /** Find a user by their unique ID. */
  findById(id: string): User | undefined {
    return this.users.get(id);
  }

  /** Find a user by their email address. */
  findByEmail(email: string): User | undefined {
    for (const user of this.users.values()) {
      if (user.email === email) {
        return user;
      }
    }
    return undefined;
  }

  /** Create a new user record and return the created user. */
  create(email: string, name: string): User {
    const user: User = {
      id: `user_${Date.now()}`,
      email,
      name,
      createdAt: new Date(),
    };
    this.users.set(user.id, user);
    return user;
  }
}
