using System;

namespace Fugle.MarketData
{
    /// <summary>
    /// Base exception for all Fugle MarketData errors
    /// </summary>
    public class FugleException : Exception
    {
        public FugleException() { }
        public FugleException(string message) : base(message) { }
        public FugleException(string message, Exception innerException) : base(message, innerException) { }
    }

    /// <summary>
    /// Authentication failed - invalid or expired API key
    /// </summary>
    public class AuthException : FugleException
    {
        public AuthException() { }
        public AuthException(string message) : base(message) { }
        public AuthException(string message, Exception innerException) : base(message, innerException) { }
    }

    /// <summary>
    /// Base class for API-related errors
    /// </summary>
    public class ApiException : FugleException
    {
        public int? StatusCode { get; set; }

        public ApiException() { }
        public ApiException(string message) : base(message) { }
        public ApiException(string message, int statusCode) : base(message)
        {
            StatusCode = statusCode;
        }
        public ApiException(string message, Exception innerException) : base(message, innerException) { }
    }

    /// <summary>
    /// Rate limit exceeded - too many requests
    /// </summary>
    public class RateLimitException : ApiException
    {
        public RateLimitException() { }
        public RateLimitException(string message) : base(message, 429) { }
        public RateLimitException(string message, Exception innerException) : base(message, innerException)
        {
            StatusCode = 429;
        }
    }

    /// <summary>
    /// Connection error - network issues or timeout
    /// </summary>
    public class ConnectionException : FugleException
    {
        public ConnectionException() { }
        public ConnectionException(string message) : base(message) { }
        public ConnectionException(string message, Exception innerException) : base(message, innerException) { }
    }

    /// <summary>
    /// Internal Fugle library error or panic recovery
    /// </summary>
    public class FugleInternalException : FugleException
    {
        public FugleInternalException() { }
        public FugleInternalException(string message) : base(message) { }
        public FugleInternalException(string message, Exception innerException) : base(message, innerException) { }
    }

    /// <summary>
    /// Helper to map error codes from native library to C# exceptions
    /// </summary>
    internal static class ErrorCodeMapper
    {
        // Error codes from cs/src/errors.rs
        private const int SUCCESS = 0;
        private const int ERROR_INVALID_ARG = -1;
        private const int ERROR_AUTH_FAILED = -2;
        private const int ERROR_RATE_LIMITED = -3;
        private const int ERROR_API_ERROR = -4;
        private const int ERROR_CONNECTION_FAILED = -5;
        private const int ERROR_TIMEOUT = -6;
        private const int ERROR_WEBSOCKET = -7;
        private const int ERROR_INTERNAL = -999;

        public static void ThrowIfError(int errorCode, string? errorMessage = null)
        {
            if (errorCode == SUCCESS)
                return;

            var message = errorMessage ?? $"Fugle API error (code: {errorCode})";

            switch (errorCode)
            {
                case ERROR_INVALID_ARG:
                    throw new ArgumentException(message);
                case ERROR_AUTH_FAILED:
                    throw new AuthException(message);
                case ERROR_RATE_LIMITED:
                    throw new RateLimitException(message);
                case ERROR_API_ERROR:
                    throw new ApiException(message);
                case ERROR_CONNECTION_FAILED:
                case ERROR_TIMEOUT:
                    throw new ConnectionException(message);
                case ERROR_WEBSOCKET:
                    throw new FugleException($"WebSocket error: {message}");
                case ERROR_INTERNAL:
                    throw new FugleInternalException(message);
                default:
                    throw new FugleException($"Unknown error code {errorCode}: {message}");
            }
        }
    }
}
