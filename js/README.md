# marketdata-js

JavaScript/TypeScript bindings for Fugle Market Data API, built with NAPI-RS.

## Installation

```bash
# From source (requires Rust toolchain)
cd marketdata-js
npm install
npm run build

# The native module is built to marketdata-js.*.node
```

## Quick Start

### REST API

```javascript
const { RestClient } = require('@fubon/marketdata-js');

// Create client with API key
const client = new RestClient('your-api-key');

// Get stock quote
const quote = client.stock.intraday.quote('2330');
console.log('TSMC Price:', quote.closePrice);

// Get futures quote
const futoptQuote = client.futopt.intraday.quote('TXFC4');
console.log('TXF Price:', futoptQuote.closePrice);
```

### WebSocket Streaming

```javascript
const { WebSocketClient } = require('@fubon/marketdata-js');

// Create client
const ws = new WebSocketClient('your-api-key');

// Register handlers
ws.stock.on('message', (data) => {
  const msg = JSON.parse(data);
  console.log('Message:', msg);
});

ws.stock.on('connect', () => {
  console.log('Connected!');
  ws.stock.subscribe({ channel: 'trades', symbol: '2330' });
});

ws.stock.on('disconnect', (reason) => {
  console.log('Disconnected:', reason);
});

ws.stock.on('error', (err) => {
  console.error('Error:', err);
});

// Connect
ws.stock.connect();

// Disconnect after 30 seconds
setTimeout(() => {
  ws.stock.disconnect();
}, 30000);
```

### TypeScript

```typescript
import { RestClient, WebSocketClient } from '@fubon/marketdata-js';

const client = new RestClient('your-api-key');
const quote = client.stock.intraday.quote('2330');
// quote is typed as Record<string, any>
```

## API Reference

### RestClient

```typescript
class RestClient {
  constructor(apiKey: string);

  stock: {
    intraday: {
      quote(symbol: string): Record<string, any>;
      ticker(symbol: string): Record<string, any>;
      candles(symbol: string): Record<string, any>;
      trades(symbol: string): Record<string, any>;
      volumes(symbol: string): Record<string, any>;
    }
  };

  futopt: {
    intraday: {
      quote(symbol: string): Record<string, any>;
      ticker(symbol: string): Record<string, any>;
      candles(symbol: string): Record<string, any>;
      trades(symbol: string): Record<string, any>;
      volumes(symbol: string): Record<string, any>;
      products(type: 'futures' | 'options'): Record<string, any>;
    }
  };
}
```

### WebSocketClient

```typescript
class WebSocketClient {
  constructor(apiKey: string);

  stock: StockWebSocketClient;
  futopt: FutOptWebSocketClient;
}

class StockWebSocketClient {
  on(event: 'message' | 'connect' | 'disconnect' | 'error', handler: Function): void;
  connect(): void;
  subscribe(options: { channel: string; symbol: string; oddLot?: boolean }): void;
  unsubscribe(subscriptionId: string): void;
  disconnect(): void;
  get isConnected(): boolean;
  get isClosed(): boolean;
}

// FutOptWebSocketClient has the same API
```

## Error Handling

```javascript
const { RestClient } = require('@fubon/marketdata-js');

const client = new RestClient('your-api-key');

try {
  const quote = client.stock.intraday.quote('INVALID');
} catch (e) {
  if (e.message.includes('[2010]')) {
    console.log('Client already closed');
  } else if (e.message.includes('[2002]')) {
    console.log('Authentication failed');
  } else {
    console.error('Error:', e.message);
  }
}
```

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1001 | InvalidSymbol | Invalid symbol format |
| 1002 | DeserializationError | Failed to parse response |
| 2001 | ConnectionError | Network connection failed |
| 2002 | AuthError | Authentication failed |
| 2003 | ApiError | API returned an error |
| 2010 | ClientClosed | Client already closed |
| 3001 | TimeoutError | Operation timed out |
| 3002 | WebSocketError | WebSocket protocol error |

## License

MIT
