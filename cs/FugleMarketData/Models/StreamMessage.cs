using System.Text.Json.Serialization;

namespace Fugle.MarketData.Models
{
    /// <summary>
    /// Generic WebSocket stream message from Fugle API
    /// </summary>
    public record StreamMessage
    {
        /// <summary>
        /// Event type (e.g., "snapshot", "data")
        /// </summary>
        [JsonPropertyName("event")]
        public string Event { get; init; } = string.Empty;

        /// <summary>
        /// Channel (e.g., "trades", "books", "candles")
        /// </summary>
        [JsonPropertyName("channel")]
        public string? Channel { get; init; }

        /// <summary>
        /// Symbol (e.g., "2330")
        /// </summary>
        [JsonPropertyName("symbol")]
        public string? Symbol { get; init; }

        /// <summary>
        /// Raw data payload - caller should deserialize based on channel type
        /// </summary>
        [JsonPropertyName("data")]
        public object? Data { get; init; }
    }
}
