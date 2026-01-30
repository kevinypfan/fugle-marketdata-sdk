using System.Text.Json.Serialization;

namespace Fugle.MarketData.Models
{
    /// <summary>
    /// Stock ticker information from Fugle API (intraday/ticker/{symbol})
    /// </summary>
    public record Ticker
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

        // === Stock info ===

        /// <summary>
        /// Stock name (Chinese)
        /// </summary>
        [JsonPropertyName("name")]
        public string? Name { get; init; }

        /// <summary>
        /// Stock name (English)
        /// </summary>
        [JsonPropertyName("nameEn")]
        public string? NameEn { get; init; }

        /// <summary>
        /// Industry category
        /// </summary>
        [JsonPropertyName("industry")]
        public string? Industry { get; init; }

        /// <summary>
        /// Security type classification
        /// </summary>
        [JsonPropertyName("securityType")]
        public string? SecurityType { get; init; }

        // === Price limits ===

        /// <summary>
        /// Reference price (previous close)
        /// </summary>
        [JsonPropertyName("referencePrice")]
        public double? ReferencePrice { get; init; }

        /// <summary>
        /// Limit up price
        /// </summary>
        [JsonPropertyName("limitUpPrice")]
        public double? LimitUpPrice { get; init; }

        /// <summary>
        /// Limit down price
        /// </summary>
        [JsonPropertyName("limitDownPrice")]
        public double? LimitDownPrice { get; init; }

        /// <summary>
        /// Previous close price
        /// </summary>
        [JsonPropertyName("previousClose")]
        public double? PreviousClose { get; init; }

        // === Trading rules ===

        /// <summary>
        /// Can day trade
        /// </summary>
        [JsonPropertyName("canDayTrade")]
        public bool CanDayTrade { get; init; }

        /// <summary>
        /// Can buy day trade
        /// </summary>
        [JsonPropertyName("canBuyDayTrade")]
        public bool CanBuyDayTrade { get; init; }

        /// <summary>
        /// Can below flat margin short sell
        /// </summary>
        [JsonPropertyName("canBelowFlatMarginShortSell")]
        public bool CanBelowFlatMarginShortSell { get; init; }

        /// <summary>
        /// Can below flat SBL short sell
        /// </summary>
        [JsonPropertyName("canBelowFlatSBLShortSell")]
        public bool CanBelowFlatSBLShortSell { get; init; }

        // === Attention flags ===

        /// <summary>
        /// Is attention stock
        /// </summary>
        [JsonPropertyName("isAttention")]
        public bool IsAttention { get; init; }

        /// <summary>
        /// Is disposition stock
        /// </summary>
        [JsonPropertyName("isDisposition")]
        public bool IsDisposition { get; init; }

        /// <summary>
        /// Is unusually recommended
        /// </summary>
        [JsonPropertyName("isUnusuallyRecommended")]
        public bool IsUnusuallyRecommended { get; init; }

        /// <summary>
        /// Is specific abnormally
        /// </summary>
        [JsonPropertyName("isSpecificAbnormally")]
        public bool IsSpecificAbnormally { get; init; }

        /// <summary>
        /// Is newly compiled
        /// </summary>
        [JsonPropertyName("isNewlyCompiled")]
        public bool IsNewlyCompiled { get; init; }
    }
}
