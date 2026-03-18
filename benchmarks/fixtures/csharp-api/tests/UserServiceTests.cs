// Unit tests for UserService
// In a real project these would use xUnit or NUnit with a test runner.
// For the benchmark fixture, compilation is the primary check.

using System.Diagnostics;
using CSharpApi.Users;

namespace CSharpApi.Tests;

/// <summary>
/// Tests for the UserService class covering user lookup and creation.
/// </summary>
public static class UserServiceTests
{
    public static void RunAll()
    {
        TestGetUserSuccess();
        TestGetUserNotFound();
        TestCreateUserSuccess();
        TestCreateUserDuplicateEmail();
        TestUserExistsReturnsTrue();
        TestUserExistsReturnsFalse();

        Console.WriteLine("All user tests passed");
    }

    private static void TestGetUserSuccess()
    {
        var repository = new UserRepository();
        var user = repository.Create("alice@example.com", "Alice");
        var service = new UserService(repository);

        var found = service.GetUser(user.Id);
        Debug.Assert(found.Email == "alice@example.com", "Expected alice's email");
    }

    private static void TestGetUserNotFound()
    {
        var repository = new UserRepository();
        var service = new UserService(repository);

        try
        {
            service.GetUser("nonexistent");
            Debug.Assert(false, "Should have thrown");
        }
        catch (InvalidOperationException)
        {
            // Expected
        }
    }

    private static void TestCreateUserSuccess()
    {
        var repository = new UserRepository();
        var service = new UserService(repository);

        var user = service.CreateUser("bob@example.com", "Bob");
        Debug.Assert(user.Email == "bob@example.com", "Expected bob's email");
        Debug.Assert(user.Name == "Bob", "Expected name Bob");
    }

    private static void TestCreateUserDuplicateEmail()
    {
        var repository = new UserRepository();
        var service = new UserService(repository);
        service.CreateUser("dup@example.com", "First");

        try
        {
            service.CreateUser("dup@example.com", "Second");
            Debug.Assert(false, "Should have thrown for duplicate email");
        }
        catch (InvalidOperationException)
        {
            // Expected
        }
    }

    private static void TestUserExistsReturnsTrue()
    {
        var repository = new UserRepository();
        var user = repository.Create("exists@example.com", "Existing");
        var service = new UserService(repository);

        Debug.Assert(service.UserExists(user.Id), "Expected user to exist");
    }

    private static void TestUserExistsReturnsFalse()
    {
        var repository = new UserRepository();
        var service = new UserService(repository);

        Debug.Assert(!service.UserExists("ghost"), "Expected user not to exist");
    }
}
