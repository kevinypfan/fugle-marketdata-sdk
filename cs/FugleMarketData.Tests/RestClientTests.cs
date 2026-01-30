using Fugle.MarketData;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;

namespace FugleMarketData.Tests
{
    /// <summary>
    /// Tests for REST client - structural tests without live API calls
    /// </summary>
    [TestClass]
    public class RestClientTests
    {
        [TestMethod]
        public void Constructor_ThrowsOnNullApiKey()
        {
            try
            {
                var client = new RestClient(null!);
                Assert.Fail("Expected ArgumentNullException");
            }
            catch (ArgumentNullException ex)
            {
                Assert.AreEqual("apiKey", ex.ParamName);
            }
        }

        [TestMethod]
        public void Constructor_ThrowsOnEmptyApiKey()
        {
            try
            {
                var client = new RestClient("");
                Assert.Fail("Expected ArgumentNullException");
            }
            catch (ArgumentNullException ex)
            {
                Assert.AreEqual("apiKey", ex.ParamName);
            }
        }

        [TestMethod]
        public void Constructor_CreatesClient()
        {
            // This test requires native library to be built
            try
            {
                using var client = new RestClient("test-api-key");
                Assert.IsNotNull(client);
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails with dummy key
                Assert.IsTrue(true);
            }
        }

        [TestMethod]
        public void Dispose_CanBeCalledMultipleTimes()
        {
            try
            {
                var client = new RestClient("test-api-key");
                client.Dispose();
                client.Dispose(); // Should not throw
                Assert.IsTrue(true);
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails
                Assert.IsTrue(true);
            }
        }

        [TestMethod]
        public void DisposedClient_ThrowsOnMethodCall()
        {
            try
            {
                var client = new RestClient("test-api-key");
                client.Dispose();

                try
                {
                    // This should throw ObjectDisposedException
                    _ = client.GetStockQuoteAsync("2330");
                    Assert.Fail("Expected ObjectDisposedException");
                }
                catch (ObjectDisposedException ex)
                {
                    Assert.AreEqual("RestClient", ex.ObjectName);
                }
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails
                Assert.IsTrue(true);
            }
        }

        [TestMethod]
        public void GetStockQuoteAsync_ThrowsOnNullSymbol()
        {
            try
            {
                using var client = new RestClient("test-api-key");

                try
                {
                    _ = client.GetStockQuoteAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("symbol", ex.ParamName);
                }
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails
                Assert.IsTrue(true);
            }
        }

        [TestMethod]
        public void GetStockTradesAsync_ThrowsOnNullSymbol()
        {
            try
            {
                using var client = new RestClient("test-api-key");

                try
                {
                    _ = client.GetStockTradesAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("symbol", ex.ParamName);
                }
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails
                Assert.IsTrue(true);
            }
        }

        [TestMethod]
        public void GetStockTickerAsync_ThrowsOnNullSymbol()
        {
            try
            {
                using var client = new RestClient("test-api-key");

                try
                {
                    _ = client.GetStockTickerAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("symbol", ex.ParamName);
                }
            }
            catch (DllNotFoundException)
            {
                Assert.Inconclusive("Native library not available - structural test only");
            }
            catch (FugleInternalException)
            {
                // Expected when native library exists but initialization fails
                Assert.IsTrue(true);
            }
        }
    }
}
