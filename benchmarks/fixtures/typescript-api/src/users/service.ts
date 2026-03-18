import { User, UserRepository } from "./repository";

/** Business logic layer for user operations. */
export class UserService {
  private repository: UserRepository;

  constructor(repository: UserRepository) {
    this.repository = repository;
  }

  /** Get a user by ID. Throws if user not found. */
  getUser(userId: string): User {
    const user = this.repository.findById(userId);
    if (!user) {
      throw new Error(`User ${userId} not found`);
    }
    return user;
  }

  /** Create a new user with the given email and name. */
  createUser(email: string, name: string): User {
    const existing = this.repository.findByEmail(email);
    if (existing) {
      throw new Error(`User with email ${email} already exists`);
    }
    return this.repository.create(email, name);
  }

  /** Check whether a user exists by their ID. */
  userExists(userId: string): boolean {
    const user = this.repository.findById(userId);
    return user !== undefined;
  }
}
