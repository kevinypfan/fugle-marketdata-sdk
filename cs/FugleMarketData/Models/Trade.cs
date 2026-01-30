using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace Fugle.MarketData.Models
{
    /// <summary>
    /// A single trade execution
    /// </summary>
    public record Trade
    {
        /// <summary>
        /// Best bid price at trade time
        /// </summary>
        [JsonPropertyName("bid")]
        public double? Bid { get; init; }

        /// <summary>
        /// Best ask price at trade time
        /// </summary>
        [JsonPropertyName("ask")]
        public double? Ask { get; init; }

        /// <summary>
        /// Trade price
        /// </summary>
        [JsonPropertyName("price")]
        public double Price { get; init; }

        /// <summary>
        /// Trade size (volume)
        /// </summary>
        [JsonPropertyName("size")]
        public long Size { get; init; }

        /// <summary>
        /// Trade timestamp (Unix milliseconds)
        /// </summary>
        [JsonPropertyName("time")]
        public long Time { get; init; }
    }

    /// <summary>
    /// Trades response from Fugle API (intraday/trades/{symbol})
    /// </summary>
    public record TradesResponse
    {
        /// <summary>
        /// Trading date (YYYY-MM-DD)
        /// </summary>
        [JsonPropertyName("date")]
        public string Date { get; init; } = string.Empty;

        /// <summary>
        /// Security type (e.g., "EQUITY", "ODDLOT")
        /// </summary>
        [JsonPropertyName("type")]
        public string? Type { get; init; }

        /// <summary>
        /// Exchange code (e.g., "TWSE", "TPEx")
        /// </summary>
        [JsonPropertyName("exchange")]
        public string? Exchange { get; init; }

        /// <summary>
        /// Market (e.g., "TSE", "OTC")
        /// </summary>
        [JsonPropertyName("market")]
        public string? Market { get; init; }

        /// <summary>
        /// Stock symbol
        /// </summary>
        [JsonPropertyName("symbol")]
        public string Symbol { get; init; } = string.Empty;

        /// <summary>
        /// List of trades
        /// </summary>
        [JsonPropertyName("data")]
        public List<Trade> Data { get; init; } = new();
    }
}
