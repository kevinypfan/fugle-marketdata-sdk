using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Fugle.MarketData.Models;
using Fugle.MarketData.Native;

namespace Fugle.MarketData
{
    /// <summary>
    /// Connection state for WebSocket client
    /// </summary>
    public enum ConnectionState
    {
        /// <summary>
        /// Not connected
        /// </summary>
        Disconnected = 0,
        /// <summary>
        /// Connecting to server
        /// </summary>
        Connecting = 1,
        /// <summary>
        /// Connected and ready
        /// </summary>
        Connected = 2,
        /// <summary>
        /// Attempting to reconnect
        /// </summary>
        Reconnecting = 3
    }

    /// <summary>
    /// WebSocket client for Fugle MarketData streaming - uses EventHandler pattern for message delivery
    /// </summary>
    public sealed class WebSocketClient : IAsyncDisposable, IDisposable
    {
        private unsafe WebSocketHandle* _handle;
        private readonly object _disposeLock = new object();
        private bool _disposed;
        private Task? _pollTask;
        private CancellationTokenSource? _pollCancellation;

        /// <summary>
        /// Raised when a message is received
        /// </summary>
        public event EventHandler<MessageEventArgs>? MessageReceived;

        /// <summary>
        /// Raised when connection is established
        /// </summary>
        public event EventHandler? Connected;

        /// <summary>
        /// Raised when connection is closed
        /// </summary>
        public event EventHandler? Disconnected;

        /// <summary>
        /// Raised when an error occurs
        /// </summary>
        public event EventHandler<ErrorEventArgs>? Error;

        /// <summary>
        /// Current connection state
        /// </summary>
        public ConnectionState State
        {
            get
            {
                ThrowIfDisposed();
                unsafe
                {
                    var stateCode = NativeMethods.fugle_ws_get_state(_handle);
                    return (ConnectionState)stateCode;
                }
            }
        }

        /// <summary>
        /// Create a new WebSocket client with API key
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        /// <exception cref="ArgumentNullException">If API key is null or empty</exception>
        /// <exception cref="FugleInternalException">If native client creation fails</exception>
        public WebSocketClient(string apiKey)
        {
            if (string.IsNullOrEmpty(apiKey))
                throw new ArgumentNullException(nameof(apiKey));

            unsafe
            {
                var apiKeyBytes = Encoding.UTF8.GetBytes(apiKey + "\0");
                fixed (byte* ptr = apiKeyBytes)
                {
                    _handle = NativeMethods.fugle_ws_client_new(ptr);
                }

                if (_handle == null)
                    throw new FugleInternalException("Failed to create WebSocket client");
            }
        }

        #region Connection Management

        /// <summary>
        /// Connect to stock endpoint (async)
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        /// <param name="cancellationToken">Cancellation token</param>
        public Task ConnectStockAsync(string apiKey, CancellationToken cancellationToken = default)
        {
            return ConnectAsync(apiKey, 0, cancellationToken);
        }

        /// <summary>
        /// Connect to FutOpt endpoint (async)
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        /// <param name="cancellationToken">Cancellation token</param>
        public Task ConnectFutOptAsync(string apiKey, CancellationToken cancellationToken = default)
        {
            return ConnectAsync(apiKey, 1, cancellationToken);
        }

        /// <summary>
        /// Connect to stock endpoint (blocking)
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        public void ConnectStock(string apiKey)
        {
            ConnectStockAsync(apiKey).GetAwaiter().GetResult();
        }

        /// <summary>
        /// Connect to FutOpt endpoint (blocking)
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        public void ConnectFutOpt(string apiKey)
        {
            ConnectFutOptAsync(apiKey).GetAwaiter().GetResult();
        }

        private Task ConnectAsync(string apiKey, int endpointType, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(apiKey))
                throw new ArgumentNullException(nameof(apiKey));

            return Task.Run(() =>
            {
                unsafe
                {
                    var apiKeyBytes = Encoding.UTF8.GetBytes(apiKey + "\0");
                    int errorCode;
                    fixed (byte* ptr = apiKeyBytes)
                    {
                        errorCode = NativeMethods.fugle_ws_connect(_handle, ptr, endpointType);
                    }

                    ErrorCodeMapper.ThrowIfError(errorCode);
                }

                // Start polling loop after successful connection
                _pollCancellation = new CancellationTokenSource();
                _pollTask = Task.Run(() => PollLoop(_pollCancellation.Token), _pollCancellation.Token);

                // Raise Connected event
                Connected?.Invoke(this, EventArgs.Empty);
            }, cancellationToken);
        }

        /// <summary>
        /// Disconnect from server (async)
        /// </summary>
        public Task DisconnectAsync(CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();

            return Task.Run(() =>
            {
                // Stop polling first
                _pollCancellation?.Cancel();

                unsafe
                {
                    var errorCode = NativeMethods.fugle_ws_disconnect(_handle);
                    ErrorCodeMapper.ThrowIfError(errorCode);
                }

                // Raise Disconnected event
                Disconnected?.Invoke(this, EventArgs.Empty);
            }, cancellationToken);
        }

        /// <summary>
        /// Disconnect from server (blocking)
        /// </summary>
        public void Disconnect()
        {
            DisconnectAsync().GetAwaiter().GetResult();
        }

        #endregion

        #region Subscription Management

        /// <summary>
        /// Subscribe to a channel for a stock symbol (async)
        /// </summary>
        /// <param name="channel">Channel name (e.g., "trades", "books", "candles")</param>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        public Task SubscribeStockAsync(string channel, string symbol, CancellationToken cancellationToken = default)
        {
            return SubscribeAsync(channel, symbol, cancellationToken);
        }

        /// <summary>
        /// Subscribe to a channel for a stock symbol (blocking)
        /// </summary>
        /// <param name="channel">Channel name (e.g., "trades", "books", "candles")</param>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        public void SubscribeStock(string channel, string symbol)
        {
            SubscribeStockAsync(channel, symbol).GetAwaiter().GetResult();
        }

        /// <summary>
        /// Subscribe to a channel for a FutOpt contract (async)
        /// </summary>
        /// <param name="channel">Channel name (e.g., "trades", "books", "candles")</param>
        /// <param name="symbol">FutOpt contract symbol (e.g., "TXFF4")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        public Task SubscribeFutOptAsync(string channel, string symbol, CancellationToken cancellationToken = default)
        {
            return SubscribeAsync(channel, symbol, cancellationToken);
        }

        /// <summary>
        /// Subscribe to a channel for a FutOpt contract (blocking)
        /// </summary>
        /// <param name="channel">Channel name (e.g., "trades", "books", "candles")</param>
        /// <param name="symbol">FutOpt contract symbol (e.g., "TXFF4")</param>
        public void SubscribeFutOpt(string channel, string symbol)
        {
            SubscribeFutOptAsync(channel, symbol).GetAwaiter().GetResult();
        }

        private Task SubscribeAsync(string channel, string symbol, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(channel))
                throw new ArgumentNullException(nameof(channel));
            if (string.IsNullOrEmpty(symbol))
                throw new ArgumentNullException(nameof(symbol));

            return Task.Run(() =>
            {
                unsafe
                {
                    var channelBytes = Encoding.UTF8.GetBytes(channel + "\0");
                    var symbolBytes = Encoding.UTF8.GetBytes(symbol + "\0");
                    int errorCode;

                    fixed (byte* channelPtr = channelBytes)
                    fixed (byte* symbolPtr = symbolBytes)
                    {
                        errorCode = NativeMethods.fugle_ws_subscribe(_handle, channelPtr, symbolPtr);
                    }

                    ErrorCodeMapper.ThrowIfError(errorCode);
                }
            }, cancellationToken);
        }

        /// <summary>
        /// Unsubscribe from a channel (async)
        /// </summary>
        /// <param name="key">Subscription key in format "channel:symbol" (e.g., "trades:2330")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        public Task UnsubscribeAsync(string key, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(key))
                throw new ArgumentNullException(nameof(key));

            return Task.Run(() =>
            {
                unsafe
                {
                    var keyBytes = Encoding.UTF8.GetBytes(key + "\0");
                    int errorCode;

                    fixed (byte* keyPtr = keyBytes)
                    {
                        errorCode = NativeMethods.fugle_ws_unsubscribe(_handle, keyPtr);
                    }

                    ErrorCodeMapper.ThrowIfError(errorCode);
                }
            }, cancellationToken);
        }

        /// <summary>
        /// Unsubscribe from a channel (blocking)
        /// </summary>
        /// <param name="key">Subscription key in format "channel:symbol" (e.g., "trades:2330")</param>
        public void Unsubscribe(string key)
        {
            UnsubscribeAsync(key).GetAwaiter().GetResult();
        }

        #endregion

        #region Polling Loop

        private const int MESSAGE_AVAILABLE = 1;
        private const int NO_MESSAGE = 0;

        private async Task PollLoop(CancellationToken cancellationToken)
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    unsafe
                    {
                        byte* messagePtr = null;
                        int result = NativeMethods.fugle_ws_poll_message(_handle, &messagePtr);

                        if (result == MESSAGE_AVAILABLE && messagePtr != null)
                        {
                            try
                            {
                                var jsonString = MarshalUtf8String(messagePtr);
                                var message = JsonSerializer.Deserialize<StreamMessage>(jsonString);

                                if (message != null)
                                {
                                    MessageReceived?.Invoke(this, new MessageEventArgs(message));
                                }
                            }
                            finally
                            {
                                NativeMethods.fugle_free_string(messagePtr);
                            }
                        }
                        else if (result < 0)
                        {
                            // Error occurred
                            try
                            {
                                ErrorCodeMapper.ThrowIfError(result);
                            }
                            catch (Exception ex)
                            {
                                Error?.Invoke(this, new ErrorEventArgs(ex));
                            }
                        }
                    }

                    // 10ms polling interval for low latency
                    await Task.Delay(10, cancellationToken).ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                    break;
                }
                catch (Exception ex)
                {
                    Error?.Invoke(this, new ErrorEventArgs(ex));
                }
            }
        }

        private static unsafe string MarshalUtf8String(byte* ptr)
        {
            if (ptr == null)
                return string.Empty;

            int length = 0;
            while (ptr[length] != 0)
                length++;

            return Encoding.UTF8.GetString(ptr, length);
        }

        #endregion

        #region IDisposable and IAsyncDisposable

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(WebSocketClient));
        }

        /// <summary>
        /// Dispose the WebSocket client asynchronously
        /// </summary>
        public async ValueTask DisposeAsync()
        {
            lock (_disposeLock)
            {
                if (_disposed)
                    return;

                _disposed = true;
            }

            // Cancel polling
            _pollCancellation?.Cancel();

            // Wait for polling task to complete
            if (_pollTask != null)
            {
                try
                {
                    await _pollTask.ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                    // Expected during cancellation
                }
            }

            _pollCancellation?.Dispose();

            // Free native handle
            unsafe
            {
                if (_handle != null)
                {
                    NativeMethods.fugle_ws_client_free(_handle);
                    _handle = null;
                }
            }
        }

        /// <summary>
        /// Dispose the WebSocket client synchronously
        /// </summary>
        public void Dispose()
        {
            DisposeAsync().AsTask().GetAwaiter().GetResult();
        }

        #endregion
    }
}
