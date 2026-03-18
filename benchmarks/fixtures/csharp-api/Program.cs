using CSharpApi.Payments;
using CSharpApi.Users;
using CSharpApi.Notifications;
using CSharpApi.Controllers;
using CSharpApi.Workers;

// Wire up dependencies (simulated DI container)
var userRepository = new UserRepository();
var userService = new UserService(userRepository);
var paymentProcessor = new PaymentProcessor("sk_live_prod_key_xxxxx");
var notificationService = new NotificationService("https://mail.example.com", "+15551234567");
var paymentService = new PaymentService(paymentProcessor, notificationService);

var orderController = new OrderController(paymentService, userService);
var refundController = new RefundController(paymentService, notificationService);
var subscriptionController = new SubscriptionController(paymentService, userService);
var retryWorker = new PaymentRetryWorker(paymentService, maxRetries: 3);

// Seed a test user
var user = userRepository.Create("alice@example.com", "Alice Smith");

Console.WriteLine($"Application started. Seeded user: {user.Id}");
