// Unit tests for UserService and UserRepository

import { UserRepository } from "../../src/users/repository";
import { UserService } from "../../src/users/service";

function testCreateUser(): void {
  const repo = new UserRepository();
  const service = new UserService(repo);
  const user = service.createUser("test@example.com", "Test User");
  console.assert(user.email === "test@example.com", "Expected email match");
  console.assert(user.name === "Test User", "Expected name match");
}

function testGetUserThrowsForMissing(): void {
  const repo = new UserRepository();
  const service = new UserService(repo);
  try {
    service.getUser("nonexistent");
    console.assert(false, "Should have thrown");
  } catch (e) {
    // Expected
  }
}

function testFindByIdReturnsUser(): void {
  const repo = new UserRepository();
  const user = repo.create("test@example.com", "Test");
  const found = repo.findById(user.id);
  console.assert(found !== undefined, "Expected to find user");
  console.assert(found!.id === user.id, "Expected ID match");
}

function testFindByEmailReturnsUser(): void {
  const repo = new UserRepository();
  repo.create("hello@example.com", "Hello");
  const found = repo.findByEmail("hello@example.com");
  console.assert(found !== undefined, "Expected to find user by email");
}

function testCreateUserRejectsDuplicate(): void {
  const repo = new UserRepository();
  const service = new UserService(repo);
  service.createUser("dup@example.com", "First");
  try {
    service.createUser("dup@example.com", "Second");
    console.assert(false, "Should have thrown for duplicate email");
  } catch (e) {
    // Expected
  }
}

// Run all tests
testCreateUser();
testGetUserThrowsForMissing();
testFindByIdReturnsUser();
testFindByEmailReturnsUser();
testCreateUserRejectsDuplicate();

console.log("All user tests passed");
