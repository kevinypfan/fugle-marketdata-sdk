namespace Fugle.MarketData
{
    /// <summary>
    /// Error codes from native library (cs/src/errors.rs)
    /// </summary>
    internal static class NativeErrorCodes
    {
        public const int SUCCESS = 0;
        public const int ERROR_INVALID_ARG = -1;
        public const int ERROR_AUTH_FAILED = -2;
        public const int ERROR_RATE_LIMITED = -3;
        public const int ERROR_API_ERROR = -4;
        public const int ERROR_CONNECTION_FAILED = -5;
        public const int ERROR_TIMEOUT = -6;
        public const int ERROR_WEBSOCKET = -7;
        public const int ERROR_INTERNAL = -999;
    }
}
