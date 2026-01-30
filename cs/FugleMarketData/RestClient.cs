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
    /// REST API client for Fugle MarketData - provides async methods for stock and FutOpt market data
    /// </summary>
    public sealed unsafe class RestClient : IDisposable
    {
        private RestClientHandle* _handle;
        private readonly object _disposeLock = new object();
        private bool _disposed;

        /// <summary>
        /// Create a new REST client with API key
        /// </summary>
        /// <param name="apiKey">Fugle API key</param>
        /// <exception cref="ArgumentNullException">If API key is null or empty</exception>
        /// <exception cref="FugleInternalException">If native client creation fails</exception>
        public RestClient(string apiKey)
        {
            if (string.IsNullOrEmpty(apiKey))
                throw new ArgumentNullException(nameof(apiKey));

            var apiKeyBytes = Encoding.UTF8.GetBytes(apiKey + "\0");
            fixed (byte* ptr = apiKeyBytes)
            {
                _handle = NativeMethods.fugle_rest_client_new(ptr);
            }

            if (_handle == null)
                throw new FugleInternalException("Failed to create REST client");
        }

        #region Stock Intraday Endpoints - Async

        /// <summary>
        /// Get real-time stock quote (async)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        /// <returns>Stock quote data</returns>
        public Task<Quote> GetStockQuoteAsync(string symbol, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(symbol))
                throw new ArgumentNullException(nameof(symbol));

            var tcs = new TaskCompletionSource<Quote>(TaskCreationOptions.RunContinuationsAsynchronously);
            var gcHandle = GCHandle.Alloc(tcs);

            try
            {
                var symbolBytes = Encoding.UTF8.GetBytes(symbol + "\0");
                fixed (byte* symbolPtr = symbolBytes)
                {
                    var callbackPtr = Marshal.GetFunctionPointerForDelegate(_callbackDelegate);
                    NativeMethods.fugle_rest_stock_quote_async(
                        _handle,
                        symbolPtr,
                        (delegate* unmanaged[Cdecl]<void*, byte*, int, void>)callbackPtr,
                        (void*)GCHandle.ToIntPtr(gcHandle)
                    );
                }

                cancellationToken.Register(() =>
                {
                    if (!tcs.Task.IsCompleted)
                        tcs.TrySetCanceled(cancellationToken);
                });

                return tcs.Task;
            }
            catch
            {
                gcHandle.Free();
                throw;
            }
        }

        /// <summary>
        /// Get intraday trades for a stock (async)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        /// <returns>Trades response with list of trades</returns>
        public Task<TradesResponse> GetStockTradesAsync(string symbol, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(symbol))
                throw new ArgumentNullException(nameof(symbol));

            var tcs = new TaskCompletionSource<TradesResponse>(TaskCreationOptions.RunContinuationsAsynchronously);
            var gcHandle = GCHandle.Alloc(tcs);

            try
            {
                var symbolBytes = Encoding.UTF8.GetBytes(symbol + "\0");
                fixed (byte* symbolPtr = symbolBytes)
                {
                    var callbackPtr = Marshal.GetFunctionPointerForDelegate(_callbackDelegate);
                    NativeMethods.fugle_rest_stock_trades_async(
                        _handle,
                        symbolPtr,
                        (delegate* unmanaged[Cdecl]<void*, byte*, int, void>)callbackPtr,
                        (void*)GCHandle.ToIntPtr(gcHandle)
                    );
                }

                cancellationToken.Register(() =>
                {
                    if (!tcs.Task.IsCompleted)
                        tcs.TrySetCanceled(cancellationToken);
                });

                return tcs.Task;
            }
            catch
            {
                gcHandle.Free();
                throw;
            }
        }

        /// <summary>
        /// Get stock ticker information (async)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <param name="cancellationToken">Cancellation token</param>
        /// <returns>Stock ticker info</returns>
        public Task<Ticker> GetStockTickerAsync(string symbol, CancellationToken cancellationToken = default)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(symbol))
                throw new ArgumentNullException(nameof(symbol));

            var tcs = new TaskCompletionSource<Ticker>(TaskCreationOptions.RunContinuationsAsynchronously);
            var gcHandle = GCHandle.Alloc(tcs);

            try
            {
                var symbolBytes = Encoding.UTF8.GetBytes(symbol + "\0");
                fixed (byte* symbolPtr = symbolBytes)
                {
                    var callbackPtr = Marshal.GetFunctionPointerForDelegate(_callbackDelegate);
                    NativeMethods.fugle_rest_stock_ticker_async(
                        _handle,
                        symbolPtr,
                        (delegate* unmanaged[Cdecl]<void*, byte*, int, void>)callbackPtr,
                        (void*)GCHandle.ToIntPtr(gcHandle)
                    );
                }

                cancellationToken.Register(() =>
                {
                    if (!tcs.Task.IsCompleted)
                        tcs.TrySetCanceled(cancellationToken);
                });

                return tcs.Task;
            }
            catch
            {
                gcHandle.Free();
                throw;
            }
        }

        #endregion

        #region Stock Intraday Endpoints - Sync

        /// <summary>
        /// Get real-time stock quote (blocking)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <returns>Stock quote data</returns>
        public Quote GetStockQuote(string symbol)
        {
            return GetStockQuoteAsync(symbol).GetAwaiter().GetResult();
        }

        /// <summary>
        /// Get intraday trades for a stock (blocking)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <returns>Trades response with list of trades</returns>
        public TradesResponse GetStockTrades(string symbol)
        {
            return GetStockTradesAsync(symbol).GetAwaiter().GetResult();
        }

        /// <summary>
        /// Get stock ticker information (blocking)
        /// </summary>
        /// <param name="symbol">Stock symbol (e.g., "2330")</param>
        /// <returns>Stock ticker info</returns>
        public Ticker GetStockTicker(string symbol)
        {
            return GetStockTickerAsync(symbol).GetAwaiter().GetResult();
        }

        #endregion

        #region Callback Handler

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        private unsafe delegate void NativeCallback(void* userData, byte* resultJson, int errorCode);

        private static readonly NativeCallback _callbackDelegate = ResultCallback;

        private static void ResultCallback(void* userData, byte* resultJson, int errorCode)
        {
            var gcHandle = GCHandle.FromIntPtr((IntPtr)userData);

            try
            {
                var tcsObj = gcHandle.Target;
                if (tcsObj == null) return;

                // Check if it's a Quote task
                if (tcsObj is TaskCompletionSource<Quote> quoteTcs)
                {
                    CompleteTask(quoteTcs, resultJson, errorCode);
                }
                // Check if it's a TradesResponse task
                else if (tcsObj is TaskCompletionSource<TradesResponse> tradesTcs)
                {
                    CompleteTask(tradesTcs, resultJson, errorCode);
                }
                // Check if it's a Ticker task
                else if (tcsObj is TaskCompletionSource<Ticker> tickerTcs)
                {
                    CompleteTask(tickerTcs, resultJson, errorCode);
                }
            }
            finally
            {
                gcHandle.Free();
            }
        }

        private static void CompleteTask<T>(TaskCompletionSource<T> tcs, byte* resultJson, int errorCode)
        {
            if (errorCode != NativeErrorCodes.SUCCESS)
            {
                try
                {
                    ErrorCodeMapper.ThrowIfError(errorCode);
                }
                catch (Exception ex)
                {
                    tcs.TrySetException(ex);
                }
                return;
            }

            if (resultJson == null)
            {
                tcs.TrySetException(new FugleInternalException("Result JSON is null"));
                return;
            }

            try
            {
                var jsonString = MarshalUtf8String(resultJson);
                var result = JsonSerializer.Deserialize<T>(jsonString);

                if (result == null)
                {
                    tcs.TrySetException(new FugleInternalException("Failed to deserialize result"));
                    return;
                }

                tcs.TrySetResult(result);
            }
            catch (Exception ex)
            {
                tcs.TrySetException(new FugleInternalException($"JSON deserialization failed: {ex.Message}", ex));
            }
            finally
            {
                // Free the string allocated by Rust
                NativeMethods.fugle_free_string(resultJson);
            }
        }

        private static string MarshalUtf8String(byte* ptr)
        {
            if (ptr == null)
                return string.Empty;

            int length = 0;
            while (ptr[length] != 0)
                length++;

            return Encoding.UTF8.GetString(ptr, length);
        }

        #endregion

        #region IDisposable

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(RestClient));
        }

        /// <summary>
        /// Dispose the REST client and free native resources
        /// </summary>
        public void Dispose()
        {
            lock (_disposeLock)
            {
                if (_disposed)
                    return;

                if (_handle != null)
                {
                    NativeMethods.fugle_rest_client_free(_handle);
                    _handle = null;
                }

                _disposed = true;
            }
        }

        #endregion
    }
}
