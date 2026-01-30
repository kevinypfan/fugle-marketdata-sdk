using System.Text.Json.Serialization;

namespace Fugle.MarketData.Models
{
    /// <summary>
    /// Price level for order book (bid/ask)
    /// </summary>
    public record PriceLevel
    {
        /// <summary>
        /// Price at this level
        /// </summary>
        [JsonPropertyName("price")]
        public double Price { get; init; }

        /// <summary>
        /// Size (volume) at this level
        /// </summary>
        [JsonPropertyName("size")]
        public long Size { get; init; }
    }

    /// <summary>
    /// Trade execution info (used in quote.LastTrade, quote.LastTrial)
    /// </summary>
    public record TradeInfo
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
        /// Trade size
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
    /// Total trading statistics
    /// </summary>
    public record TotalStats
    {
        /// <summary>
        /// Total trade value
        /// </summary>
        [JsonPropertyName("tradeValue")]
        public double TradeValue { get; init; }

        /// <summary>
        /// Total trade volume
        /// </summary>
        [JsonPropertyName("tradeVolume")]
        public long TradeVolume { get; init; }

        /// <summary>
        /// Volume traded at bid
        /// </summary>
        [JsonPropertyName("tradeVolumeAtBid")]
        public long? TradeVolumeAtBid { get; init; }

        /// <summary>
        /// Volume traded at ask
        /// </summary>
        [JsonPropertyName("tradeVolumeAtAsk")]
        public long? TradeVolumeAtAsk { get; init; }

        /// <summary>
        /// Number of transactions
        /// </summary>
        [JsonPropertyName("transaction")]
        public long? Transaction { get; init; }

        /// <summary>
        /// Timestamp
        /// </summary>
        [JsonPropertyName("time")]
        public long? Time { get; init; }
    }

    /// <summary>
    /// Trading halt status
    /// </summary>
    public record TradingHalt
    {
        /// <summary>
        /// Whether trading is halted
        /// </summary>
        [JsonPropertyName("isHalted")]
        public bool IsHalted { get; init; }

        /// <summary>
        /// Halt timestamp
        /// </summary>
        [JsonPropertyName("time")]
        public long? Time { get; init; }
    }
}
