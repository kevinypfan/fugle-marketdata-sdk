using Fugle.MarketData;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace FugleMarketData.Tests
{
    /// <summary>
    /// Tests for exception hierarchy and error code mapping
    /// </summary>
    [TestClass]
    public class ExceptionTests
    {
        [TestMethod]
        public void ExceptionHierarchy_IsCorrect()
        {
            // Verify inheritance structure
            Assert.IsTrue(typeof(FugleException).IsSubclassOf(typeof(Exception)));
            Assert.IsTrue(typeof(AuthException).IsSubclassOf(typeof(FugleException)));
            Assert.IsTrue(typeof(ApiException).IsSubclassOf(typeof(FugleException)));
            Assert.IsTrue(typeof(RateLimitException).IsSubclassOf(typeof(ApiException)));
            Assert.IsTrue(typeof(ConnectionException).IsSubclassOf(typeof(FugleException)));
            Assert.IsTrue(typeof(FugleInternalException).IsSubclassOf(typeof(FugleException)));
        }

        [TestMethod]
        public void RateLimitException_HasCorrectStatusCode()
        {
            var ex = new RateLimitException("Rate limit exceeded");
            Assert.AreEqual(429, ex.StatusCode);
        }

        [TestMethod]
        public void ApiException_SupportsStatusCode()
        {
            var ex = new ApiException("API error", 500);
            Assert.AreEqual(500, ex.StatusCode);
        }

        [TestMethod]
        public void FugleException_SupportsInnerException()
        {
            var inner = new InvalidOperationException("Inner error");
            var ex = new FugleException("Outer error", inner);
            Assert.AreEqual(inner, ex.InnerException);
        }

        [TestMethod]
        public void AuthException_CanBeCreatedWithMessage()
        {
            var ex = new AuthException("Authentication failed");
            Assert.AreEqual("Authentication failed", ex.Message);
        }

        [TestMethod]
        public void ConnectionException_CanBeCreatedWithMessage()
        {
            var ex = new ConnectionException("Connection timeout");
            Assert.AreEqual("Connection timeout", ex.Message);
        }

        [TestMethod]
        public void FugleInternalException_CanBeCreatedWithMessage()
        {
            var ex = new FugleInternalException("Internal panic");
            Assert.AreEqual("Internal panic", ex.Message);
        }
    }
}
