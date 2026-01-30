using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace Fugle.MarketData.Models
{
    /// <summary>
    /// Real-time stock quote from Fugle API (intraday/quote/{symbol})
    /// </summary>
    public record Quote
    {
        // === Response metadata ===

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
        /// Stock symbol (e.g., "2330")
        /// </summary>
        [JsonPropertyName("symbol")]
        public string Symbol { get; init; } = string.Empty;

        /// <summary>
        /// Stock name
        /// </summary>
        [JsonPropertyName("name")]
        public string? Name { get; init; }

        // === OHLC prices with timestamps ===

        /// <summary>
        /// Open price
        /// </summary>
        [JsonPropertyName("openPrice")]
        public double? OpenPrice { get; init; }

        /// <summary>
        /// Open time (Unix ms)
        /// </summary>
        [JsonPropertyName("openTime")]
        public long? OpenTime { get; init; }

        /// <summary>
        /// High price
        /// </summary>
        [JsonPropertyName("highPrice")]
        public double? HighPrice { get; init; }

        /// <summary>
        /// High time (Unix ms)
        /// </summary>
        [JsonPropertyName("highTime")]
        public long? HighTime { get; init; }

        /// <summary>
        /// Low price
        /// </summary>
        [JsonPropertyName("lowPrice")]
        public double? LowPrice { get; init; }

        /// <summary>
        /// Low time (Unix ms)
        /// </summary>
        [JsonPropertyName("lowTime")]
        public long? LowTime { get; init; }

        /// <summary>
        /// Close price
        /// </summary>
        [JsonPropertyName("closePrice")]
        public double? ClosePrice { get; init; }

        /// <summary>
        /// Close time (Unix ms)
        /// </summary>
        [JsonPropertyName("closeTime")]
        public long? CloseTime { get; init; }

        // === Current trading info ===

        /// <summary>
        /// Last traded price
        /// </summary>
        [JsonPropertyName("lastPrice")]
        public double? LastPrice { get; init; }

        /// <summary>
        /// Last traded size
        /// </summary>
        [JsonPropertyName("lastSize")]
        public long? LastSize { get; init; }

        /// <summary>
        /// Average price
        /// </summary>
        [JsonPropertyName("avgPrice")]
        public double? AvgPrice { get; init; }

        /// <summary>
        /// Price change from previous close
        /// </summary>
        [JsonPropertyName("change")]
        public double? Change { get; init; }

        /// <summary>
        /// Percentage change from previous close
        /// </summary>
        [JsonPropertyName("changePercent")]
        public double? ChangePercent { get; init; }

        /// <summary>
        /// Price amplitude (high - low) / previous close * 100
        /// </summary>
        [JsonPropertyName("amplitude")]
        public double? Amplitude { get; init; }

        // === Order book ===

        /// <summary>
        /// Bid price levels
        /// </summary>
        [JsonPropertyName("bids")]
        public List<PriceLevel> Bids { get; init; } = new();

        /// <summary>
        /// Ask price levels
        /// </summary>
        [JsonPropertyName("asks")]
        public List<PriceLevel> Asks { get; init; } = new();

        // === Aggregated stats ===

        /// <summary>
        /// Total trading statistics
        /// </summary>
        [JsonPropertyName("total")]
        public TotalStats? Total { get; init; }

        /// <summary>
        /// Last trade info
        /// </summary>
        [JsonPropertyName("lastTrade")]
        public TradeInfo? LastTrade { get; init; }

        /// <summary>
        /// Last trial (simulated matching) info
        /// </summary>
        [JsonPropertyName("lastTrial")]
        public TradeInfo? LastTrial { get; init; }

        /// <summary>
        /// Trading halt status
        /// </summary>
        [JsonPropertyName("tradingHalt")]
        public TradingHalt? TradingHalt { get; init; }

        // === Limit price flags ===

        /// <summary>
        /// Is at limit down price
        /// </summary>
        [JsonPropertyName("isLimitDownPrice")]
        public bool IsLimitDownPrice { get; init; }

        /// <summary>
        /// Is at limit up price
        /// </summary>
        [JsonPropertyName("isLimitUpPrice")]
        public bool IsLimitUpPrice { get; init; }

        /// <summary>
        /// Is limit down bid
        /// </summary>
        [JsonPropertyName("isLimitDownBid")]
        public bool IsLimitDownBid { get; init; }

        /// <summary>
        /// Is limit up bid
        /// </summary>
        [JsonPropertyName("isLimitUpBid")]
        public bool IsLimitUpBid { get; init; }

        /// <summary>
        /// Is limit down ask
        /// </summary>
        [JsonPropertyName("isLimitDownAsk")]
        public bool IsLimitDownAsk { get; init; }

        /// <summary>
        /// Is limit up ask
        /// </summary>
        [JsonPropertyName("isLimitUpAsk")]
        public bool IsLimitUpAsk { get; init; }

        /// <summary>
        /// Is limit down halt
        /// </summary>
        [JsonPropertyName("isLimitDownHalt")]
        public bool IsLimitDownHalt { get; init; }

        /// <summary>
        /// Is limit up halt
        /// </summary>
        [JsonPropertyName("isLimitUpHalt")]
        public bool IsLimitUpHalt { get; init; }

        // === Trading session flags ===

        /// <summary>
        /// Is in trial (simulated matching) period
        /// </summary>
        [JsonPropertyName("isTrial")]
        public bool IsTrial { get; init; }

        /// <summary>
        /// Is delayed open
        /// </summary>
        [JsonPropertyName("isDelayedOpen")]
        public bool IsDelayedOpen { get; init; }

        /// <summary>
        /// Is delayed close
        /// </summary>
        [JsonPropertyName("isDelayedClose")]
        public bool IsDelayedClose { get; init; }

        /// <summary>
        /// Is continuous trading
        /// </summary>
        [JsonPropertyName("isContinuous")]
        public bool IsContinuous { get; init; }

        /// <summary>
        /// Is market open
        /// </summary>
        [JsonPropertyName("isOpen")]
        public bool IsOpen { get; init; }

        /// <summary>
        /// Is market closed
        /// </summary>
        [JsonPropertyName("isClose")]
        public bool IsClose { get; init; }

        /// <summary>
        /// Last updated timestamp (Unix ms)
        /// </summary>
        [JsonPropertyName("lastUpdated")]
        public long? LastUpdated { get; init; }
    }
}
