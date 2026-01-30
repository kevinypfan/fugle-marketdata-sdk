using System;
using Fugle.MarketData.Models;

namespace Fugle.MarketData
{
    /// <summary>
    /// Event args for WebSocket message received events
    /// </summary>
    public class MessageEventArgs : EventArgs
    {
        /// <summary>
        /// The received stream message
        /// </summary>
        public StreamMessage Message { get; }

        public MessageEventArgs(StreamMessage message)
        {
            Message = message;
        }
    }

    /// <summary>
    /// Event args for WebSocket error events
    /// </summary>
    public class ErrorEventArgs : EventArgs
    {
        /// <summary>
        /// The exception that occurred
        /// </summary>
        public Exception Exception { get; }

        public ErrorEventArgs(Exception exception)
        {
            Exception = exception;
        }
    }
}
