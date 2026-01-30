/**
 * TypeScript type definitions for @fugle/marketdata SDK
 *
 * This file contains all interface and type definitions for REST API responses
 * and WebSocket events. These types provide full IDE autocomplete support.
 */

// ============================================================================
// Common Types
// ============================================================================

/** Price level for order book (bid/ask) */
export interface PriceLevel {
  /** Price at this level */
  price: number;
  /** Size (volume) at this level */
  size: number;
}

/** Trade execution info */
export interface TradeInfo {
  /** Best bid price at trade time */
  bid?: number;
  /** Best ask price at trade time */
  ask?: number;
  /** Trade price */
  price: number;
  /** Trade size */
  size: number;
  /** Trade timestamp (Unix milliseconds) */
  time: number;
}

/** Total trading statistics */
export interface TotalStats {
  /** Total trade value */
  tradeValue: number;
  /** Total trade volume */
  tradeVolume: number;
  /** Volume traded at bid */
  tradeVolumeAtBid?: number;
  /** Volume traded at ask */
  tradeVolumeAtAsk?: number;
  /** Number of transactions */
  transaction?: number;
  /** Timestamp */
  time?: number;
}

/** Trading halt status */
export interface TradingHalt {
  /** Whether trading is halted */
  isHalted: boolean;
  /** Halt timestamp */
  time?: number;
}

// ============================================================================
// REST Response Types - Stock/FutOpt Intraday
// ============================================================================

/**
 * Quote response from intraday/quote/{symbol}
 *
 * Contains real-time price, volume, and order book data.
 */
export interface QuoteResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Security type (e.g., "EQUITY", "ODDLOT") */
  type?: string;
  /** Exchange code (e.g., "TWSE", "TPEx", "TAIFEX") */
  exchange?: string;
  /** Market (e.g., "TSE", "OTC") */
  market?: string;
  /** Symbol (e.g., "2330" for stock, "TXFC4" for futures) */
  symbol: string;
  /** Security name */
  name?: string;

  // OHLC prices with timestamps
  /** Open price */
  openPrice?: number;
  /** Open time (Unix milliseconds) */
  openTime?: number;
  /** High price */
  highPrice?: number;
  /** High time (Unix milliseconds) */
  highTime?: number;
  /** Low price */
  lowPrice?: number;
  /** Low time (Unix milliseconds) */
  lowTime?: number;
  /** Close price */
  closePrice?: number;
  /** Close time (Unix milliseconds) */
  closeTime?: number;

  // Current trading info
  /** Last traded price */
  lastPrice?: number;
  /** Last traded size */
  lastSize?: number;
  /** Average price */
  avgPrice?: number;
  /** Price change from previous close */
  change?: number;
  /** Percentage change from previous close */
  changePercent?: number;
  /** Price amplitude */
  amplitude?: number;

  // Order book
  /** Bid price levels */
  bids: PriceLevel[];
  /** Ask price levels */
  asks: PriceLevel[];

  // Aggregated stats
  /** Total trading statistics */
  total?: TotalStats;
  /** Last trade info */
  lastTrade?: TradeInfo;
  /** Last trial (simulated matching) info */
  lastTrial?: TradeInfo;
  /** Trading halt status */
  tradingHalt?: TradingHalt;

  // Limit price flags
  /** Is at limit down price */
  isLimitDownPrice: boolean;
  /** Is at limit up price */
  isLimitUpPrice: boolean;
  /** Is limit down bid */
  isLimitDownBid: boolean;
  /** Is limit up bid */
  isLimitUpBid: boolean;
  /** Is limit down ask */
  isLimitDownAsk: boolean;
  /** Is limit up ask */
  isLimitUpAsk: boolean;
  /** Is limit down halt */
  isLimitDownHalt: boolean;
  /** Is limit up halt */
  isLimitUpHalt: boolean;

  // Trading session flags
  /** Is in trial (simulated matching) period */
  isTrial: boolean;
  /** Is delayed open */
  isDelayedOpen: boolean;
  /** Is delayed close */
  isDelayedClose: boolean;
  /** Is continuous trading */
  isContinuous: boolean;
  /** Is market open */
  isOpen: boolean;
  /** Is market closed */
  isClose: boolean;

  /** Last updated timestamp (Unix milliseconds) */
  lastUpdated?: number;
}

/**
 * Ticker response from intraday/ticker/{symbol}
 *
 * Contains static security information and trading rules.
 */
export interface TickerResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Security type (e.g., "EQUITY", "ODDLOT") */
  type?: string;
  /** Exchange code */
  exchange?: string;
  /** Market */
  market?: string;
  /** Symbol */
  symbol: string;

  // Stock info
  /** Stock name (Chinese) */
  name?: string;
  /** Stock name (English) */
  nameEn?: string;
  /** Industry category */
  industry?: string;
  /** Security type classification */
  securityType?: string;

  // Price limits
  /** Reference price (previous close) */
  referencePrice?: number;
  /** Limit up price */
  limitUpPrice?: number;
  /** Limit down price */
  limitDownPrice?: number;
  /** Previous close price */
  previousClose?: number;

  // Trading rules
  /** Can day trade */
  canDayTrade: boolean;
  /** Can buy day trade */
  canBuyDayTrade: boolean;
  /** Can below flat margin short sell */
  canBelowFlatMarginShortSell: boolean;
  /** Can below flat SBL short sell */
  canBelowFlatSBLShortSell: boolean;

  // Attention flags
  /** Is attention stock */
  isAttention: boolean;
  /** Is disposition stock */
  isDisposition: boolean;
  /** Is unusually recommended */
  isUnusuallyRecommended: boolean;
  /** Is specific abnormally */
  isSpecificAbnormally: boolean;
  /** Is newly compiled */
  isNewlyCompiled: boolean;

  // Trading parameters
  /** Matching interval (seconds) */
  matchingInterval?: number;
  /** Security status */
  securityStatus?: string;
  /** Board lot size */
  boardLot?: number;
  /** Trading currency */
  tradingCurrency?: string;

  // Warrant/ETN specific
  /** Exercise price */
  exercisePrice?: number;
  /** Exercised volume */
  exercisedVolume?: number;
  /** Cancelled volume */
  cancelledVolume?: number;
  /** Remaining volume */
  remainingVolume?: number;
  /** Exercise ratio */
  exerciseRatio?: number;
  /** Cap price */
  capPrice?: number;
  /** Floor price */
  floorPrice?: number;
  /** Maturity date */
  maturityDate?: string;

  // Session times
  /** Open time */
  openTime?: string;
  /** Close time */
  closeTime?: string;
}

/** A single intraday candlestick bar */
export interface IntradayCandle {
  /** Open price */
  open: number;
  /** High price */
  high: number;
  /** Low price */
  low: number;
  /** Close price */
  close: number;
  /** Volume */
  volume: number;
  /** Average price (VWAP for the candle period) */
  average?: number;
  /** Candle timestamp (Unix milliseconds) */
  time: number;
}

/**
 * Candles response from intraday/candles/{symbol}
 *
 * Contains OHLCV candlestick data.
 */
export interface CandlesResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Security type */
  type?: string;
  /** Exchange code */
  exchange?: string;
  /** Market */
  market?: string;
  /** Symbol */
  symbol: string;
  /** Timeframe (e.g., "1", "5", "10", "15", "30", "60") */
  timeframe?: string;
  /** Candle data */
  data: IntradayCandle[];
}

/** A single trade execution */
export interface Trade {
  /** Best bid price at trade time */
  bid?: number;
  /** Best ask price at trade time */
  ask?: number;
  /** Trade price */
  price: number;
  /** Trade size (volume) */
  size: number;
  /** Trade timestamp (Unix milliseconds) */
  time: number;
}

/**
 * Trades response from intraday/trades/{symbol}
 *
 * Contains recent trade executions.
 */
export interface TradesResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Security type */
  type?: string;
  /** Exchange code */
  exchange?: string;
  /** Market */
  market?: string;
  /** Symbol */
  symbol: string;
  /** Trade data */
  data: Trade[];
}

/** Volume at a specific price level */
export interface VolumeAtPrice {
  /** Price level */
  price: number;
  /** Total volume at this price */
  volume: number;
  /** Volume traded at bid */
  volumeAtBid?: number;
  /** Volume traded at ask */
  volumeAtAsk?: number;
}

/**
 * Volumes response from intraday/volumes/{symbol}
 *
 * Contains volume profile data at each price level.
 */
export interface VolumesResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Security type */
  type?: string;
  /** Exchange code */
  exchange?: string;
  /** Market */
  market?: string;
  /** Symbol */
  symbol: string;
  /** Volume data at each price level */
  data: VolumeAtPrice[];
}

// ============================================================================
// REST Response Types - FutOpt Specific
// ============================================================================

/** Contract type for futures/options */
export type ContractType = 'I' | 'R' | 'B' | 'C' | 'S' | 'E';

/** FutOpt type */
export type FutOptType = 'FUTURE' | 'OPTION';

/** A single product entry in FutOpt products response */
export interface FutOptProduct {
  /** Product type (FUTURE/OPTION) */
  type: string;
  /** Exchange code */
  exchange: string;
  /** Contract symbol */
  symbol: string;
  /** Contract name */
  name: string;
  /** Underlying symbol */
  underlyingSymbol: string;
  /** Contract type */
  contractType: string;
  /** Contract size */
  contractSize: number;
  /** Status code */
  statusCode: string;
  /** Trading currency */
  tradingCurrency: string;
  /** Whether quote is acceptable */
  quoteAcceptable: boolean;
  /** Start date */
  startDate: string;
  /** Whether block trade is allowed */
  canBlockTrade: boolean;
  /** Expiry type */
  expiryType: string;
  /** Underlying type */
  underlyingType: string;
  /** Market close group */
  marketCloseGroup: number;
  /** End session */
  endSession: number;
}

/**
 * Products response from futopt/intraday/products
 *
 * Contains available futures/options contracts.
 */
export interface ProductsResponse {
  /** Trading date (YYYY-MM-DD) */
  date: string;
  /** Product type */
  type: string;
  /** Trading session */
  session: string;
  /** Contract type filter applied */
  contractType: string;
  /** Status filter applied */
  status: string;
  /** Product list */
  data: FutOptProduct[];
}

// ============================================================================
// WebSocket Types
// ============================================================================

/** WebSocket message payload */
export interface WebSocketMessage {
  /** Event type */
  event: string;
  /** Message data */
  data: Record<string, unknown>;
  /** Channel name */
  channel?: string;
  /** Symbol */
  symbol?: string;
}

/** Stock WebSocket channel types */
export type StockChannel = 'trades' | 'books' | 'candles' | 'aggregates' | 'indices';

/** FutOpt WebSocket channel types */
export type FutOptChannel = 'trades' | 'books' | 'candles' | 'aggregates';

/**
 * Subscribe options for stock WebSocket
 */
export interface StockSubscribeOptions {
  /** Channel to subscribe to */
  channel: StockChannel;
  /** Stock symbol */
  symbol: string;
  /** Include intraday odd lot data */
  intradayOddLot?: boolean;
}

/**
 * Subscribe options for FutOpt WebSocket
 */
export interface FutOptSubscribeOptions {
  /** Channel to subscribe to */
  channel: FutOptChannel;
  /** Contract symbol */
  symbol: string;
  /** Include after-hours data */
  afterHours?: boolean;
}

/**
 * Event map for typed WebSocket callbacks
 */
export interface WebSocketEventMap {
  /** Market data message received */
  message: (data: string) => void;
  /** Connected to WebSocket server */
  connect: (info: string) => void;
  /** Disconnected from WebSocket server */
  disconnect: (reason: string) => void;
  /** Reconnecting to WebSocket server */
  reconnect: (info: string) => void;
  /** Error occurred */
  error: (error: string) => void;
}

/** Event names for WebSocket */
export type WebSocketEvent = keyof WebSocketEventMap;
