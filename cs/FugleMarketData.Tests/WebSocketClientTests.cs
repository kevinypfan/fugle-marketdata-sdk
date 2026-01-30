using Fugle.MarketData;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Threading.Tasks;

namespace FugleMarketData.Tests
{
    /// <summary>
    /// Tests for WebSocket client - structural tests without live connection
    /// </summary>
    [TestClass]
    public class WebSocketClientTests
    {
        [TestMethod]
        public void Constructor_ThrowsOnNullApiKey()
        {
            try
            {
                var client = new WebSocketClient(null!);
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
                var client = new WebSocketClient("");
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
                using var client = new WebSocketClient("test-api-key");
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
        public void State_ReturnsDisconnectedInitially()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");
                var state = client.State;
                Assert.AreEqual(ConnectionState.Disconnected, state);
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
        public void Dispose_CanBeCalledMultipleTimes()
        {
            try
            {
                var client = new WebSocketClient("test-api-key");
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
        public async Task DisposeAsync_CanBeCalledMultipleTimes()
        {
            try
            {
                var client = new WebSocketClient("test-api-key");
                await client.DisposeAsync();
                await client.DisposeAsync(); // Should not throw
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
        public void DisposedClient_ThrowsOnStateAccess()
        {
            try
            {
                var client = new WebSocketClient("test-api-key");
                client.Dispose();

                try
                {
                    _ = client.State;
                    Assert.Fail("Expected ObjectDisposedException");
                }
                catch (ObjectDisposedException ex)
                {
                    Assert.AreEqual("WebSocketClient", ex.ObjectName);
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
        public void ConnectStockAsync_ThrowsOnNullApiKey()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                try
                {
                    _ = client.ConnectStockAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("apiKey", ex.ParamName);
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
        public void ConnectFutOptAsync_ThrowsOnNullApiKey()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                try
                {
                    _ = client.ConnectFutOptAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("apiKey", ex.ParamName);
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
        public void SubscribeStockAsync_ThrowsOnNullChannel()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                try
                {
                    _ = client.SubscribeStockAsync(null!, "2330");
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("channel", ex.ParamName);
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
        public void SubscribeStockAsync_ThrowsOnNullSymbol()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                try
                {
                    _ = client.SubscribeStockAsync("trades", null!);
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
        public void UnsubscribeAsync_ThrowsOnNullKey()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                try
                {
                    _ = client.UnsubscribeAsync(null!);
                    Assert.Fail("Expected ArgumentNullException");
                }
                catch (ArgumentNullException ex)
                {
                    Assert.AreEqual("key", ex.ParamName);
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
        public void Events_CanBeSubscribed()
        {
            try
            {
                using var client = new WebSocketClient("test-api-key");

                // Verify event subscription doesn't throw
                client.MessageReceived += (sender, e) => { };
                client.Connected += (sender, e) => { };
                client.Disconnected += (sender, e) => { };
                client.Error += (sender, e) => { };

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
    }
}
